#!/bin/bash

# Script to test the Feedback API

API_URL="${API_URL:-http://localhost:8080}"
TOKEN_FILE=".token"

echo "🧪 Testing Feedback API..."
echo ""

# Check if token file exists
if [ ! -f "$TOKEN_FILE" ]; then
    echo "❌ Token file not found. Run ./scripts/get-token.sh --save first."
    exit 1
fi

TOKEN=$(cat $TOKEN_FILE)

# Test 1: Health Check
echo "1️⃣  Testing Health Check..."
RESPONSE=$(curl -s "$API_URL/health")
echo "Response: $RESPONSE"
echo ""

# Test 2: Submit a Rating Feedback
echo "2️⃣  Submitting Rating Feedback (Visio Service)..."
RESPONSE=$(curl -s -X POST "$API_URL/api/v1/feedbacks" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "service": "visio",
    "feedback_type": "rating",
    "rating": 4,
    "comment": "Good call quality!",
    "context": {
      "call_id": "call_123",
      "duration": 1800
    }
  }')
echo "Response: $RESPONSE"
FEEDBACK_ID=$(echo $RESPONSE | grep -o '"id":"[^"]*' | cut -d'"' -f4)
echo ""

# Test 3: Submit a Thumbs Feedback
echo "3️⃣  Submitting Thumbs Feedback (Chatbot Service)..."
RESPONSE=$(curl -s -X POST "$API_URL/api/v1/feedbacks" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "service": "chatbot",
    "feedback_type": "thumbs",
    "thumbs_up": true,
    "comment": "Great response!",
    "context": {
      "message_id": "msg_456"
    }
  }')
echo "Response: $RESPONSE"
echo ""

# Test 4: Submit an NPS Feedback
echo "4️⃣  Submitting NPS Feedback (Console Service)..."
RESPONSE=$(curl -s -X POST "$API_URL/api/v1/feedbacks" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "service": "console",
    "feedback_type": "nps",
    "rating": 9,
    "comment": "Excellent platform!"
  }')
echo "Response: $RESPONSE"
echo ""

# Test 5: Get All Feedbacks
echo "5️⃣  Getting All Feedbacks..."
RESPONSE=$(curl -s "$API_URL/api/v1/feedbacks?limit=5" \
  -H "Authorization: Bearer $TOKEN")
echo "Response: $RESPONSE"
echo ""

# Test 6: Get Specific Feedback
if [ ! -z "$FEEDBACK_ID" ]; then
    echo "6️⃣  Getting Feedback by ID ($FEEDBACK_ID)..."
    RESPONSE=$(curl -s "$API_URL/api/v1/feedbacks/$FEEDBACK_ID" \
      -H "Authorization: Bearer $TOKEN")
    echo "Response: $RESPONSE"
    echo ""
fi

# Test 7: Get Statistics
echo "7️⃣  Getting Statistics..."
RESPONSE=$(curl -s "$API_URL/api/v1/feedbacks/stats" \
  -H "Authorization: Bearer $TOKEN")
echo "Response: $RESPONSE"
echo ""

# Test 8: Get Statistics for Specific Service
echo "8️⃣  Getting Statistics for Visio Service..."
RESPONSE=$(curl -s "$API_URL/api/v1/feedbacks/stats?service=visio" \
  -H "Authorization: Bearer $TOKEN")
echo "Response: $RESPONSE"
echo ""

# Test 9: Export as JSON
echo "9️⃣  Exporting Feedbacks as JSON..."
RESPONSE=$(curl -s "$API_URL/api/v1/feedbacks/export?format=json&service=visio" \
  -H "Authorization: Bearer $TOKEN")
echo "Response (first 500 chars): ${RESPONSE:0:500}..."
echo ""

# Test 10: Check Metrics
echo "🔟 Checking Prometheus Metrics..."
RESPONSE=$(curl -s "$API_URL/metrics" | grep "feedback_total")
echo "Metrics (feedback_total):"
echo "$RESPONSE"
echo ""

echo "✅ All tests completed!"
echo ""
echo "📊 Check the following:"
echo "  - Grafana Dashboard:  http://localhost:3000"
echo "  - Webhook Monitor:    http://localhost:8081"
echo "  - Prometheus Metrics: http://localhost:9090"
