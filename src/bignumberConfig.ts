import BigNumber from 'bignumber.js';

BigNumber.config({
  EXPONENTIAL_AT: [-1e9, 1e9],
  // bignumber.js 10 made invalid input throw by default. Sage parses user
  // input with it and checks isNaN(), so keep returning NaN instead.
  STRICT: false,
});
