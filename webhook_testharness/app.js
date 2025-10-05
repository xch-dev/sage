// Load environment variables from .env file
require('dotenv').config();

var createError = require('http-errors');
var express = require('express');
var path = require('path');
var cookieParser = require('cookie-parser');
var logger = require('morgan');
const bodyParser = require('body-parser');
const https = require('https');
const fs = require('fs');
var indexRouter = require('./routes/index');

var app = express();

// view engine setup
app.set('views', path.join(__dirname, 'views'));
app.set('view engine', 'pug');

app.use(logger('dev'));
app.use(express.json());
app.use(express.urlencoded({ extended: false }));
app.use(cookieParser());
app.use(express.static(path.join(__dirname, 'public')));
app.use(bodyParser.json());
app.use('/', indexRouter);

app.post('/sage_hook', (req, res) => {
  console.log(req.body);
  res.status(200).end();
});

// Helper function to create mTLS agent
function createMTLSAgent() {
  const certPath = process.env.CLIENT_CERT_PATH;
  const keyPath = process.env.CLIENT_KEY_PATH;
  const cert = process.env.CLIENT_CERT;
  const key = process.env.CLIENT_KEY;

  let certData, keyData;

  if (certPath && keyPath) {
    // Read from files
    try {
      certData = fs.readFileSync(certPath, 'utf8');
      keyData = fs.readFileSync(keyPath, 'utf8');
    } catch (err) {
      throw new Error(`Failed to read certificate files: ${err.message}`);
    }
  } else if (cert && key) {
    // Use environment variables directly
    certData = cert;
    keyData = key;
  } else {
    throw new Error(
      'Either CLIENT_CERT_PATH/CLIENT_KEY_PATH or CLIENT_CERT/CLIENT_KEY environment variables must be set',
    );
  }

  return new https.Agent({
    cert: certData,
    key: keyData,
    rejectUnauthorized: false, // Set to true if you want to verify the server certificate
  });
}

// Proxy endpoint for registering webhook with mTLS
app.post('/proxy/register_webhook', (req, res) => {
  const agent = createMTLSAgent();

  const postData = JSON.stringify(req.body);

  const options = {
    hostname: 'localhost',
    port: 9257,
    path: '/register_webhook',
    method: 'POST',
    agent: agent,
    headers: {
      'Content-Type': 'application/json',
      'Content-Length': Buffer.byteLength(postData),
    },
  };

  const proxyReq = https.request(options, (proxyRes) => {
    let data = '';

    proxyRes.on('data', (chunk) => {
      data += chunk;
    });

    proxyRes.on('end', () => {
      try {
        const jsonData = JSON.parse(data);
        res.json(jsonData);
      } catch (e) {
        res.status(proxyRes.statusCode).send(data);
      }
    });
  });

  proxyReq.on('error', (err) => {
    console.error('Proxy request error:', err);
    res
      .status(500)
      .json({ error: 'Proxy request failed', details: err.message });
  });

  proxyReq.write(postData);
  proxyReq.end();
});

// Proxy endpoint for unregistering webhook with mTLS
app.post('/proxy/unregister_webhook', (req, res) => {
  const agent = createMTLSAgent();

  const postData = JSON.stringify(req.body);

  const options = {
    hostname: 'localhost',
    port: 9257,
    path: '/unregister_webhook',
    method: 'POST',
    agent: agent,
    headers: {
      'Content-Type': 'application/json',
      'Content-Length': Buffer.byteLength(postData),
    },
  };

  const proxyReq = https.request(options, (proxyRes) => {
    let data = '';

    proxyRes.on('data', (chunk) => {
      data += chunk;
    });

    proxyRes.on('end', () => {
      try {
        const jsonData = JSON.parse(data);
        res.json(jsonData);
      } catch (e) {
        res.status(proxyRes.statusCode).send(data);
      }
    });
  });

  proxyReq.on('error', (err) => {
    console.error('Proxy request error:', err);
    res
      .status(500)
      .json({ error: 'Proxy request failed', details: err.message });
  });

  proxyReq.write(postData);
  proxyReq.end();
});

// catch 404 and forward to error handler
app.use(function (req, res, next) {
  next(createError(404));
});

// error handler
app.use(function (err, req, res, next) {
  // set locals, only providing error in development
  res.locals.message = err.message;
  res.locals.error = req.app.get('env') === 'development' ? err : {};

  // render the error page
  res.status(err.status || 500);
  res.render('error');
});

module.exports = app;
