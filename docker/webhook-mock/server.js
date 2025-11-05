const express = require('express');
const app = express();
const port = 8081;

app.use(express.json());

// Store webhooks in memory
let webhooks = [];

// Serve HTML page
app.get('/', (req, res) => {
  res.send(`
    <!DOCTYPE html>
    <html>
    <head>
      <title>Webhook Monitor</title>
      <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          min-height: 100vh;
          padding: 20px;
        }
        .container {
          max-width: 1200px;
          margin: 0 auto;
        }
        h1 {
          color: white;
          text-align: center;
          margin-bottom: 30px;
          font-size: 2.5em;
          text-shadow: 2px 2px 4px rgba(0,0,0,0.2);
        }
        .stats {
          display: flex;
          gap: 20px;
          margin-bottom: 30px;
        }
        .stat-card {
          flex: 1;
          background: white;
          padding: 20px;
          border-radius: 12px;
          box-shadow: 0 10px 30px rgba(0,0,0,0.2);
          text-align: center;
        }
        .stat-number {
          font-size: 3em;
          font-weight: bold;
          color: #667eea;
        }
        .stat-label {
          color: #666;
          margin-top: 10px;
          font-size: 0.9em;
        }
        .webhooks {
          display: grid;
          gap: 20px;
        }
        .webhook-card {
          background: white;
          border-radius: 12px;
          padding: 25px;
          box-shadow: 0 10px 30px rgba(0,0,0,0.2);
          animation: slideIn 0.3s ease-out;
        }
        @keyframes slideIn {
          from { opacity: 0; transform: translateY(-20px); }
          to { opacity: 1; transform: translateY(0); }
        }
        .webhook-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 15px;
          padding-bottom: 15px;
          border-bottom: 2px solid #f0f0f0;
        }
        .webhook-event {
          font-size: 1.2em;
          font-weight: bold;
          color: #667eea;
        }
        .webhook-time {
          color: #999;
          font-size: 0.9em;
        }
        .webhook-service {
          display: inline-block;
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          color: white;
          padding: 5px 15px;
          border-radius: 20px;
          font-size: 0.9em;
          font-weight: 500;
          margin-top: 10px;
        }
        .webhook-type {
          display: inline-block;
          background: #4CAF50;
          color: white;
          padding: 5px 15px;
          border-radius: 20px;
          font-size: 0.9em;
          margin-left: 10px;
        }
        .webhook-details {
          margin-top: 15px;
          display: grid;
          gap: 10px;
        }
        .detail-row {
          display: flex;
          gap: 10px;
        }
        .detail-label {
          font-weight: bold;
          color: #666;
          min-width: 120px;
        }
        .detail-value {
          color: #333;
          flex: 1;
        }
        .rating {
          color: #FFA500;
          font-size: 1.2em;
        }
        .thumbs-up { color: #4CAF50; font-size: 1.5em; }
        .thumbs-down { color: #f44336; font-size: 1.5em; }
        .comment-box {
          background: #f9f9f9;
          padding: 15px;
          border-radius: 8px;
          margin-top: 10px;
          font-style: italic;
          color: #555;
        }
        .no-webhooks {
          text-align: center;
          color: white;
          font-size: 1.2em;
          margin-top: 50px;
        }
      </style>
    </head>
    <body>
      <div class="container">
        <h1>🎯 Feedback Webhook Monitor</h1>
        <div class="stats">
          <div class="stat-card">
            <div class="stat-number" id="total-count">0</div>
            <div class="stat-label">Total Webhooks</div>
          </div>
          <div class="stat-card">
            <div class="stat-number" id="recent-count">0</div>
            <div class="stat-label">Last 5 Minutes</div>
          </div>
        </div>
        <div class="webhooks" id="webhooks">
          <div class="no-webhooks">Waiting for webhooks...</div>
        </div>
      </div>
      <script>
        function formatDate(dateString) {
          return new Date(dateString).toLocaleString();
        }

        function getRatingStars(rating) {
          return '⭐'.repeat(rating);
        }

        function updateWebhooks() {
          fetch('/webhooks')
            .then(res => res.json())
            .then(data => {
              const container = document.getElementById('webhooks');
              document.getElementById('total-count').textContent = data.length;

              const fiveMinutesAgo = Date.now() - 5 * 60 * 1000;
              const recentCount = data.filter(w => new Date(w.timestamp) > fiveMinutesAgo).length;
              document.getElementById('recent-count').textContent = recentCount;

              if (data.length === 0) {
                container.innerHTML = '<div class="no-webhooks">Waiting for webhooks...</div>';
                return;
              }

              container.innerHTML = data.map(webhook => {
                const fb = webhook.payload.feedback;
                let feedbackDisplay = '';

                if (fb.rating !== null) {
                  feedbackDisplay = \`<div class="rating">\${getRatingStars(fb.rating)} (\${fb.rating}/5)</div>\`;
                }
                if (fb.thumbs_up !== null) {
                  feedbackDisplay = fb.thumbs_up ?
                    '<span class="thumbs-up">👍 Thumbs Up</span>' :
                    '<span class="thumbs-down">👎 Thumbs Down</span>';
                }

                return \`
                  <div class="webhook-card">
                    <div class="webhook-header">
                      <div>
                        <div class="webhook-event">\${webhook.payload.event}</div>
                        <span class="webhook-service">\${fb.service}</span>
                        <span class="webhook-type">\${fb.feedback_type}</span>
                      </div>
                      <div class="webhook-time">\${formatDate(webhook.timestamp)}</div>
                    </div>
                    <div class="webhook-details">
                      <div class="detail-row">
                        <span class="detail-label">User ID:</span>
                        <span class="detail-value">\${fb.user_id}</span>
                      </div>
                      <div class="detail-row">
                        <span class="detail-label">Feedback:</span>
                        <span class="detail-value">\${feedbackDisplay}</span>
                      </div>
                      \${fb.comment ? \`<div class="comment-box">💬 "\${fb.comment}"</div>\` : ''}
                    </div>
                  </div>
                \`;
              }).join('');
            });
        }

        // Update every 2 seconds
        setInterval(updateWebhooks, 2000);
        updateWebhooks();
      </script>
    </body>
    </html>
  `);
});

// Webhook endpoint
app.post('/webhook', (req, res) => {
  const webhook = {
    timestamp: new Date().toISOString(),
    payload: req.body
  };

  webhooks.unshift(webhook);

  // Keep only last 50 webhooks
  if (webhooks.length > 50) {
    webhooks = webhooks.slice(0, 50);
  }

  console.log('Webhook received:', webhook);
  res.status(200).json({ message: 'Webhook received' });
});

// Get all webhooks
app.get('/webhooks', (req, res) => {
  res.json(webhooks);
});

app.listen(port, '0.0.0.0', () => {
  console.log(`Webhook mock server running at http://0.0.0.0:${port}`);
});
