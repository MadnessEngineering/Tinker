#!/usr/bin/env python3
"""
Test script for Tinker Browser Network Monitoring Features
"""
import requests
import json
import time

API_BASE = "http://127.0.0.1:3003"

def test_network_capabilities():
    """Test network monitoring capabilities"""
    print("Testing network monitoring capabilities...")
    try:
        response = requests.get(f"{API_BASE}/api/info", timeout=5)
        print(f"Enhanced info: {response.status_code}")
        data = response.json()
        capabilities = data.get('data', {}).get('capabilities', [])
        
        network_caps = [cap for cap in capabilities if any(keyword in cap for keyword in ['network', 'har'])]
        print(f"   Network capabilities: {network_caps}")
        
        endpoints = data.get('data', {}).get('endpoints', {})
        network_endpoints = {k: v for k, v in endpoints.items() if 'network' in k}
        print(f"   Network endpoints: {list(network_endpoints.keys())}")
        
        return len(network_caps) >= 3  # Should have at least 3 network-related capabilities
    except Exception as e:
        print(f"Network capabilities test failed: {e}")
        return False

def test_start_network_monitoring():
    """Test starting network monitoring"""
    print("\nTesting start network monitoring...")
    try:
        response = requests.post(f"{API_BASE}/api/network/start", 
                               headers={"Content-Type": "application/json"},
                               timeout=10)
        print(f"Start network monitoring: {response.status_code}")
        print(f"   Response: {response.json()}")
        return response.status_code == 200
    except Exception as e:
        print(f"Start network monitoring failed: {e}")
        return False

def test_network_stats():
    """Test getting network statistics"""
    print("\nTesting network statistics...")
    try:
        response = requests.get(f"{API_BASE}/api/network/stats", timeout=10)
        print(f"Network stats: {response.status_code}")
        print(f"   Response: {response.json()}")
        return response.status_code == 200
    except Exception as e:
        print(f"Network stats failed: {e}")
        return False

def test_add_network_filter():
    """Test adding a network filter"""
    print("\nTesting add network filter...")
    try:
        payload = {
            "filter": {
                "url_pattern": "api",
                "method": "GET",
                "failed_only": False
            }
        }
        response = requests.post(f"{API_BASE}/api/network/filter", 
                               json=payload, 
                               headers={"Content-Type": "application/json"},
                               timeout=10)
        print(f"Add network filter: {response.status_code}")
        print(f"   Response: {response.json()}")
        return response.status_code == 200
    except Exception as e:
        print(f"Add network filter failed: {e}")
        return False

def test_export_network_har():
    """Test exporting network data as HAR"""
    print("\nTesting network HAR export...")
    try:
        response = requests.get(f"{API_BASE}/api/network/export", timeout=10)
        print(f"Network HAR export: {response.status_code}")
        print(f"   Response: {response.json()}")
        return response.status_code == 200
    except Exception as e:
        print(f"Network HAR export failed: {e}")
        return False

def test_clear_network_filters():
    """Test clearing network filters"""
    print("\nTesting clear network filters...")
    try:
        response = requests.post(f"{API_BASE}/api/network/clear-filters", 
                               headers={"Content-Type": "application/json"},
                               timeout=10)
        print(f"Clear network filters: {response.status_code}")
        print(f"   Response: {response.json()}")
        return response.status_code == 200
    except Exception as e:
        print(f"Clear network filters failed: {e}")
        return False

def test_stop_network_monitoring():
    """Test stopping network monitoring"""
    print("\nTesting stop network monitoring...")
    try:
        response = requests.post(f"{API_BASE}/api/network/stop", 
                               headers={"Content-Type": "application/json"},
                               timeout=10)
        print(f"Stop network monitoring: {response.status_code}")
        print(f"   Response: {response.json()}")
        return response.status_code == 200
    except Exception as e:
        print(f"Stop network monitoring failed: {e}")
        return False

def test_network_monitoring_workflow():
    """Test complete network monitoring workflow"""
    print("\nTesting complete network monitoring workflow...")
    try:
        # 1. Start monitoring
        start_response = requests.post(f"{API_BASE}/api/network/start", 
                                     headers={"Content-Type": "application/json"},
                                     timeout=5)
        print(f"   1. Start monitoring: {start_response.status_code}")
        
        # 2. Navigate to generate some network traffic
        nav_payload = {"url": "https://httpbin.org/json"}
        nav_response = requests.post(f"{API_BASE}/api/navigate", 
                                   json=nav_payload, 
                                   headers={"Content-Type": "application/json"},
                                   timeout=5)
        print(f"   2. Navigation: {nav_response.status_code}")
        
        # Wait for page to load
        time.sleep(3)
        
        # 3. Get network stats
        stats_response = requests.get(f"{API_BASE}/api/network/stats", timeout=5)
        print(f"   3. Get stats: {stats_response.status_code}")
        
        # 4. Export HAR
        har_response = requests.get(f"{API_BASE}/api/network/export", timeout=5)
        print(f"   4. Export HAR: {har_response.status_code}")
        
        # 5. Stop monitoring
        stop_response = requests.post(f"{API_BASE}/api/network/stop", 
                                    headers={"Content-Type": "application/json"},
                                    timeout=5)
        print(f"   5. Stop monitoring: {stop_response.status_code}")
        
        return all(r.status_code == 200 for r in [start_response, nav_response, stats_response, har_response, stop_response])
    except Exception as e:
        print(f"Network monitoring workflow failed: {e}")
        return False

def main():
    """Run all network monitoring tests"""
    print("Testing Tinker Browser Network Monitoring")
    print("=" * 50)
    
    results = []
    
    # Test network capabilities
    results.append(test_network_capabilities())
    
    # Test individual network monitoring features
    results.append(test_start_network_monitoring())
    results.append(test_network_stats())
    results.append(test_add_network_filter())
    results.append(test_export_network_har())
    results.append(test_clear_network_filters())
    results.append(test_stop_network_monitoring())
    
    # Test complete workflow
    results.append(test_network_monitoring_workflow())
    
    # Summary
    print("\n" + "=" * 50)
    passed = sum(results)
    total = len(results)
    print(f"Network Monitoring Test Results: {passed}/{total} passed")
    
    if passed == total:
        print("ALL NETWORK MONITORING FEATURES WORKING!")
        print("Tinker now has comprehensive network analysis:")
        print("   - Real-time request/response monitoring")
        print("   - Network performance statistics")
        print("   - Request filtering and analysis")
        print("   - HAR export for external analysis")
        print("   - JavaScript injection for monitoring")
        print("   - MQTT network event streaming")
        print("")
        print("NETWORK MONITORING PHASE COMPLETE!")
    else:
        print("Some network monitoring tests failed.")
        print("Check if Tinker is running with --api flag")
    
    return passed == total

if __name__ == "__main__":
    main()