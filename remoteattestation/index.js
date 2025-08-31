const express = require('express');
const { TappdClient } = require('@phala/dstack-sdk');

const app = express();
app.use(express.json());

const PORT = process.env.PORT || 3001;

// Root endpoint for basic health check
app.get('/', (req, res) => {
  res.json({ status: 'ok', message: 'RA Report server is running' });
});

// Get TEE information
app.get('/info', async (req, res) => {
  try {
    const client = new TappdClient();
    const info = await client.info();
    res.json(info);
  } catch (error) {
    console.error('Error getting TEE info:', error);
    res.status(500).json({ error: error.message || 'Failed to get TEE info' });
  }
});

// Generate attestation quote
app.post('/generate-quote', async (req, res) => {
  try {
    const userData = req.body.userData || 'default-user-data';
    const hashAlgorithm = req.body.hashAlgorithm || 'sha256';
    
    console.log(`Generating quote with userData: ${userData}, algorithm: ${hashAlgorithm}`);
    
    const client = new TappdClient();
    const quoteResult = await client.tdxQuote( userData, 'raw');
    const rtmrs = quoteResult.replayRtmrs();
    
    res.json({
      quote: quoteResult.quote,
      eventLog: quoteResult.event_log,
      rtmrs: rtmrs
    });
  } catch (error) {
    console.error('Error generating quote:', error);
    res.status(500).json({ error: error.message || 'Failed to generate quote' });
  }
});

// Start the server
app.listen(PORT, () => {
  console.log(`RA Report server listening on port ${PORT}`);
}); 