#!/usr/bin/env python3
"""
Test script for Tinker Browser Visual Testing API
"""
import requests
import json
import time

API_BASE = "http://127.0.0.1:3003"

def test_screenshot():
    """Test screenshot capture"""
    print("📸 Testing screenshot capture...")
    try:
        payload = {
            "options": {
                "format": "PNG",
                "quality": 90
            }
        }
        response = requests.post(f"{API_BASE}/api/screenshot", 
                               json=payload, 
                               headers={"Content-Type": "application/json"},
                               timeout=10)
        print(f"✅ Screenshot: {response.status_code}")
        print(f"   Response: {response.json()}")
        return response.status_code == 200
    except Exception as e:
        print(f"❌ Screenshot failed: {e}")
        return False

def test_create_baseline():
    """Test baseline creation"""
    print("\n📏 Testing baseline creation...")
    try:
        payload = {
            "test_name": "homepage_test",
            "options": {
                "format": "PNG"
            }
        }
        response = requests.post(f"{API_BASE}/api/visual/baseline", 
                               json=payload, 
                               headers={"Content-Type": "application/json"},
                               timeout=10)
        print(f"✅ Create baseline: {response.status_code}")
        print(f"   Response: {response.json()}")
        return response.status_code == 200
    except Exception as e:
        print(f"❌ Baseline creation failed: {e}")
        return False

def test_visual_regression():
    """Test visual regression testing"""
    print("\n🔍 Testing visual regression...")
    try:
        payload = {
            "test_name": "homepage_test",
            "tolerance": 0.05,  # 5% tolerance
            "options": {
                "format": "PNG"
            }
        }
        response = requests.post(f"{API_BASE}/api/visual/test", 
                               json=payload, 
                               headers={"Content-Type": "application/json"},
                               timeout=10)
        print(f"✅ Visual test: {response.status_code}")
        print(f"   Response: {response.json()}")
        return response.status_code == 200
    except Exception as e:
        print(f"❌ Visual test failed: {e}")
        return False

def test_enhanced_capabilities():
    """Test enhanced browser info with visual capabilities"""
    print("\n🔍 Testing enhanced browser capabilities...")
    try:
        response = requests.get(f"{API_BASE}/api/info", timeout=5)
        print(f"✅ Enhanced info: {response.status_code}")
        data = response.json()
        capabilities = data.get('data', {}).get('capabilities', [])
        
        visual_caps = [cap for cap in capabilities if 'visual' in cap or 'screenshot' in cap]
        print(f"   Visual capabilities: {visual_caps}")
        
        endpoints = data.get('data', {}).get('endpoints', {})
        visual_endpoints = {k: v for k, v in endpoints.items() if 'screenshot' in k or 'visual' in k}
        print(f"   Visual endpoints: {visual_endpoints}")
        
        return len(visual_caps) > 0
    except Exception as e:
        print(f"❌ Enhanced info failed: {e}")
        return False

def test_navigation_with_visual():
    """Test navigation followed by screenshot"""
    print("\n🧭 Testing navigation + screenshot workflow...")
    try:
        # Navigate to a different page
        nav_payload = {"url": "https://www.rust-lang.org"}
        nav_response = requests.post(f"{API_BASE}/api/navigate", 
                                   json=nav_payload, 
                                   headers={"Content-Type": "application/json"},
                                   timeout=5)
        print(f"✅ Navigation: {nav_response.status_code}")
        
        # Wait a moment for navigation
        time.sleep(2)
        
        # Take screenshot of new page
        screenshot_payload = {"options": {"format": "JPEG", "quality": 80}}
        screenshot_response = requests.post(f"{API_BASE}/api/screenshot", 
                                          json=screenshot_payload, 
                                          headers={"Content-Type": "application/json"},
                                          timeout=10)
        print(f"✅ Post-navigation screenshot: {screenshot_response.status_code}")
        
        return nav_response.status_code == 200 and screenshot_response.status_code == 200
    except Exception as e:
        print(f"❌ Navigation + screenshot workflow failed: {e}")
        return False

def main():
    """Run all visual testing API tests"""
    print("🎨 Testing Tinker Browser Visual Testing API")
    print("=" * 60)
    
    results = []
    
    # Test enhanced capabilities first
    results.append(test_enhanced_capabilities())
    
    # Test basic screenshot functionality
    results.append(test_screenshot())
    
    # Test baseline creation
    results.append(test_create_baseline())
    
    # Test visual regression testing
    results.append(test_visual_regression())
    
    # Test navigation + screenshot workflow
    results.append(test_navigation_with_visual())
    
    # Summary
    print("\n" + "=" * 60)
    passed = sum(results)
    total = len(results)
    print(f"📊 Visual Testing Results: {passed}/{total} passed")
    
    if passed == total:
        print("🎉 All visual testing features working perfectly!")
        print("🚀 Tinker now has complete visual testing capabilities:")
        print("   • Screenshot capture with multiple formats")
        print("   • Baseline creation for regression testing")
        print("   • Visual comparison with configurable tolerance")
        print("   • Integration with browser navigation")
    else:
        print("⚠️  Some visual tests failed. Check if Tinker is running with --api flag")
    
    return passed == total

if __name__ == "__main__":
    main()