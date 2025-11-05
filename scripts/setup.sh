#!/bin/bash

set -e

echo "🚀 Setting up Feedback API Development Environment..."
echo ""

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed. Please install Docker first."
    exit 1
fi

# Check if Docker Compose is installed
if ! command -v docker-compose &> /dev/null; then
    echo "❌ Docker Compose is not installed. Please install Docker Compose first."
    exit 1
fi

# Create .env file if it doesn't exist
if [ ! -f .env ]; then
    echo "📝 Creating .env file..."
    cp .env.example .env
    echo "✅ .env file created. Please review and update it if needed."
else
    echo "✅ .env file already exists."
fi

# Start Docker Compose services
echo ""
echo "🐳 Starting Docker Compose services..."
docker-compose up -d

echo ""
echo "⏳ Waiting for services to be ready..."
sleep 10

# Check if services are running
echo ""
echo "🔍 Checking service health..."

if docker-compose ps | grep -q "feedback-postgres.*Up"; then
    echo "✅ PostgreSQL is running"
else
    echo "❌ PostgreSQL failed to start"
fi

if docker-compose ps | grep -q "keycloak.*Up"; then
    echo "✅ Keycloak is running"
else
    echo "❌ Keycloak failed to start"
fi

if docker-compose ps | grep -q "feedback-api.*Up"; then
    echo "✅ Feedback API is running"
else
    echo "❌ Feedback API failed to start"
fi

if docker-compose ps | grep -q "grafana.*Up"; then
    echo "✅ Grafana is running"
else
    echo "❌ Grafana failed to start"
fi

echo ""
echo "🎉 Setup complete!"
echo ""
echo "📋 Available services:"
echo "  - API:             http://localhost:8080"
echo "  - Health Check:    http://localhost:8080/health"
echo "  - Metrics:         http://localhost:8080/metrics"
echo "  - Grafana:         http://localhost:3000 (admin/admin)"
echo "  - Prometheus:      http://localhost:9090"
echo "  - Keycloak:        http://localhost:8180 (admin/admin)"
echo "  - Webhook Monitor: http://localhost:8081"
echo "  - Feedback UI:     http://localhost:8082"
echo ""
echo "📖 Next steps:"
echo "  1. Get a JWT token from Keycloak"
echo "  2. Test the API with curl or Postman"
echo "  3. View metrics in Grafana"
echo "  4. Check webhook events at http://localhost:8081"
echo ""
echo "🛠️  Useful commands:"
echo "  - View logs:     docker-compose logs -f"
echo "  - Stop services: docker-compose down"
echo "  - Restart API:   docker-compose restart feedback-api"
echo ""
