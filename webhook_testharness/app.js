require('dotenv').config();

var createError = require('http-errors');
var express = require('express');
var path = require('path');
var cookieParser = require('cookie-parser');
var logger = require('morgan');
const bodyParser = require('body-parser');
const https = require('https');
const fs = require('fs');
const crypto = require('crypto');
var indexRouter = require('./routes/index');

const sseConnections = new Set();

let webhookSecret = null;

var app = express();

app.set('views', path.join(__dirname, 'views'));
app.set('view engine', 'pug');

app.use(logger('dev'));
app.use(express.urlencoded({ extended: false }));
app.use(cookieParser());
app.use(express.static(path.join(__dirname, 'public')));

// Webhook endpoint with HMAC verification (must be before bodyParser.json())
app.post(
  '/sage_hook',
  bodyParser.raw({ type: 'application/json' }),
  (req, res) => {
    const signature = req.headers['x-webhook-signature'];
    let verificationStatus = 'No signature required';
    let isValid = true;

    // Verify HMAC signature if secret is configured
    if (webhookSecret) {
      if (!signature) {
        verificationStatus = '❌ FAILED: Missing signature header';
        isValid = false;
        console.error('Webhook verification failed: Missing signature');
      } else {
        try {
          // Extract the signature from "sha256=<hex>" format
          const signatureParts = signature.split('=');
          if (signatureParts.length !== 2 || signatureParts[0] !== 'sha256') {
            verificationStatus = '❌ FAILED: Invalid signature format';
            isValid = false;
            console.error('Invalid signature format:', signature);
          } else {
            const receivedSignature = signatureParts[1];

            const hmac = crypto.createHmac('sha256', webhookSecret);
            hmac.update(req.body);
            const expectedSignature = hmac.digest('hex');

            if (
              crypto.timingSafeEqual(
                Buffer.from(receivedSignature),
                Buffer.from(expectedSignature),
              )
            ) {
              verificationStatus = '✅ VERIFIED';
              console.log('Webhook signature verified successfully');
            } else {
              verificationStatus = '❌ FAILED: Signature mismatch';
              isValid = false;
              console.error('Webhook verification failed: Signature mismatch');
              console.error('Expected:', expectedSignature);
              console.error('Received:', receivedSignature);
            }
          }
        } catch (error) {
          verificationStatus = `❌ FAILED: ${error.message}`;
          isValid = false;
          console.error('Webhook verification error:', error);
        }
      }
    }

    const parsedBody = JSON.parse(req.body.toString());
    console.log('Webhook received:', parsedBody);
    console.log('Verification status:', verificationStatus);

    const eventData = {
      id: Date.now(),
      event: 'webhook',
      data: JSON.stringify({
        timestamp: new Date().toISOString(),
        body: parsedBody,
        verification: verificationStatus,
        signature: signature || 'none',
      }),
    };

    broadcastSSEEvent(eventData);

    if (isValid) {
      res.status(200).end();
    } else {
      res.status(401).json({ error: 'Signature verification failed' });
    }
  },
);

// Endpoint to sync secret from browser cookie to server memory
//for demonstration purposes only
app.post('/sync_secret', bodyParser.json(), (req, res) => {
  const { secret } = req.body;
  if (secret) {
    webhookSecret = secret;
    res.json({ status: 'ok', message: 'Secret synced' });
  } else {
    webhookSecret = null;
    res.json({ status: 'ok', message: 'Secret cleared' });
  }
});

app.use(bodyParser.json());
app.use('/', indexRouter);

// SSE endpoint for webhook events
app.get('/events', (req, res) => {
  // Set SSE headers
  res.writeHead(200, {
    'Content-Type': 'text/event-stream',
    'Cache-Control': 'no-cache',
    Connection: 'keep-alive',
    'Access-Control-Allow-Origin': '*',
    'Access-Control-Allow-Headers': 'Cache-Control',
  });

  // Send initial connection event
  res.write(
    `data: ${JSON.stringify({
      id: Date.now(),
      event: 'connected',
      data: 'Connected to webhook event stream',
    })}\n\n`,
  );

  sseConnections.add(res);

  // Handle client disconnect
  req.on('close', () => {
    sseConnections.delete(res);
    console.log('SSE client disconnected');
  });

  console.log('SSE client connected');
});

function broadcastSSEEvent(eventData) {
  const message = `id: ${eventData.id}\nevent: ${eventData.event}\ndata: ${eventData.data}\n\n`;

  sseConnections.forEach((res) => {
    try {
      res.write(message);
    } catch (error) {
      console.error('Error sending SSE message:', error);
      sseConnections.delete(res);
    }
  });
}

function createMTLSAgent() {
  const certPath = process.env.CLIENT_CERT_PATH;
  const keyPath = process.env.CLIENT_KEY_PATH;
  const cert = process.env.CLIENT_CERT;
  const key = process.env.CLIENT_KEY;

  let certData, keyData;

  if (certPath && keyPath) {
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

  // Store the secret if provided
  if (req.body.secret) {
    webhookSecret = req.body.secret;
    console.log('Webhook secret stored for verification');
  } else {
    webhookSecret = null;
    console.log('No webhook secret provided');
  }

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

  webhookSecret = null;
  console.log('Webhook secret cleared');

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
