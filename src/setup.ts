import BigNumber from 'bignumber.js';
import { installConsoleCapture } from './lib/consoleCapture';

BigNumber.config({ EXPONENTIAL_AT: [-1e9, 1e9] });

installConsoleCapture();
