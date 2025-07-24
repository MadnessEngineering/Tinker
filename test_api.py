#!/usr/bin/env python3
"""
Quick test script for Tinker Browser API
"""
import requests
import json
import asyncio
import websockets

API_BASE = "http://127.0.0.1:3003"
WS_URL = "ws://127.0.0.1:3003/ws"

def test_health():
    """Test the health endpoint"""
    print("🔍 Testing health endpoint...")
    try:
        response = requests.get(f"{API_BASE}/health", timeout=5)
        print(f"✅ Health check: {response.status_code}")
        print(f"   Response: {response.json()}")
        return response.status_code == 200
    except Exception as e:
        print(f"❌ Health check failed: {e}")
        return False

def test_browser_info():
    """Test the browser info endpoint"""
    print("\n🔍 Testing browser info endpoint...")
    try:
        response = requests.get(f"{API_BASE}/api/info", timeout=5)
        print(f"✅ Browser info: {response.status_code}")
        data = response.json()
        print(f"   Browser: {data['data']['name']} v{data['data']['version']}")
        print(f"   Capabilities: {data['data']['capabilities']}")
        return response.status_code == 200
    except Exception as e:
        print(f"❌ Browser info failed: {e}")
        return False

def test_navigation():
    """Test navigation endpoint"""
    print("\n🔍 Testing navigation...")
    try:
        payload = {"url": "https://www.google.com"}
        response = requests.post(f"{API_BASE}/api/navigate", 
                               json=payload, 
                               headers={"Content-Type": "application/json"},
                               timeout=5)
        print(f"✅ Navigate: {response.status_code}")
        print(f"   Response: {response.json()}")
        return response.status_code == 200
    except Exception as e:
        print(f"❌ Navigation failed: {e}")
        return False

def test_tab_creation():
    """Test tab creation endpoint"""
    print("\n🔍 Testing tab creation...")
    try:
        payload = {"url": "https://github.com"}
        response = requests.post(f"{API_BASE}/api/tabs", 
                               json=payload, 
                               headers={"Content-Type": "application/json"},
                               timeout=5)
        print(f"✅ Create tab: {response.status_code}")
        print(f"   Response: {response.json()}")
        return response.status_code == 200
    except Exception as e:
        print(f"❌ Tab creation failed: {e}")
        return False

async def test_websocket():
    """Test WebSocket connection"""
    print("\n🔍 Testing WebSocket connection...")
    try:
        async with websockets.connect(WS_URL, timeout=5) as websocket:
            print("✅ WebSocket connected!")
            
            # Send a test command
            command = {
                "navigate": {
                    "url": "https://www.rust-lang.org"
                }
            }
            await websocket.send(json.dumps(command))
            print(f"✅ Sent command: {command}")
            
            # Try to receive a message (with timeout)
            try:
                message = await asyncio.wait_for(websocket.recv(), timeout=2.0)
                print(f"✅ Received: {message}")
            except asyncio.TimeoutError:
                print("⏰ No immediate response (expected)")
            
            return True
    except Exception as e:
        print(f"❌ WebSocket test failed: {e}")
        return False

def main():
    """Run all API tests"""
    print("🚀 Testing Tinker Browser API")
    print("=" * 50)
    
    results = []
    
    # HTTP API tests
    results.append(test_health())
    results.append(test_browser_info())
    results.append(test_navigation())
    results.append(test_tab_creation())
    
    # WebSocket test
    try:
        result = asyncio.run(test_websocket())
        results.append(result)
    except Exception as e:
        print(f"❌ WebSocket async test failed: {e}")
        results.append(False)
    
    # Summary
    print("\n" + "=" * 50)
    passed = sum(results)
    total = len(results)
    print(f"📊 Test Results: {passed}/{total} passed")
    
    if passed == total:
        print("🎉 All tests passed! Tinker API is working perfectly!")
    else:
        print("⚠️  Some tests failed. Check if Tinker is running with --api flag")
    
    return passed == total

if __name__ == "__main__":
    main()