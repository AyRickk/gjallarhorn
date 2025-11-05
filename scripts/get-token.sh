#!/bin/bash

# Script to get a JWT token from Keycloak for testing

KEYCLOAK_URL="${KEYCLOAK_URL:-http://localhost:8180}"
REALM="${REALM:-master}"
CLIENT_ID="${CLIENT_ID:-admin-cli}"
USERNAME="${USERNAME:-admin}"
PASSWORD="${PASSWORD:-admin}"

echo "🔑 Getting JWT token from Keycloak..."
echo ""
echo "Configuration:"
echo "  Keycloak URL: $KEYCLOAK_URL"
echo "  Realm:        $REALM"
echo "  Client ID:    $CLIENT_ID"
echo "  Username:     $USERNAME"
echo ""

TOKEN_RESPONSE=$(curl -s -X POST "$KEYCLOAK_URL/realms/$REALM/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "client_id=$CLIENT_ID" \
  -d "username=$USERNAME" \
  -d "password=$PASSWORD" \
  -d "grant_type=password")

if [ $? -ne 0 ]; then
    echo "❌ Failed to connect to Keycloak"
    exit 1
fi

ACCESS_TOKEN=$(echo $TOKEN_RESPONSE | grep -o '"access_token":"[^"]*' | cut -d'"' -f4)

if [ -z "$ACCESS_TOKEN" ]; then
    echo "❌ Failed to get access token"
    echo "Response: $TOKEN_RESPONSE"
    exit 1
fi

echo "✅ Token obtained successfully!"
echo ""
echo "Access Token:"
echo "$ACCESS_TOKEN"
echo ""
echo "Example usage with curl:"
echo ""
echo "curl -H 'Authorization: Bearer $ACCESS_TOKEN' http://localhost:8080/api/v1/feedbacks"
echo ""

# Optionally save to file
if [ "$1" == "--save" ]; then
    echo "$ACCESS_TOKEN" > .token
    echo "💾 Token saved to .token file"
    echo ""
    echo "Use it with:"
    echo "curl -H \"Authorization: Bearer \$(cat .token)\" http://localhost:8080/api/v1/feedbacks"
fi
