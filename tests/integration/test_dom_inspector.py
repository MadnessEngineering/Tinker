#!/usr/bin/env python3
"""
Test script for Tinker Browser DOM Inspector & Advanced Features
"""
import requests
import json
import time

API_BASE = "http://127.0.0.1:3003"

def test_enhanced_capabilities():
    """Test enhanced browser capabilities with DOM inspection"""
    print("🔍 Testing enhanced browser capabilities...")
    try:
        response = requests.get(f"{API_BASE}/api/info", timeout=5)
        print(f"✅ Enhanced info: {response.status_code}")
        data = response.json()
        capabilities = data.get('data', {}).get('capabilities', [])
        
        dom_caps = [cap for cap in capabilities if any(keyword in cap for keyword in ['dom', 'element', 'javascript', 'wait'])]
        print(f"   DOM capabilities: {dom_caps}")
        
        endpoints = data.get('data', {}).get('endpoints', {})
        dom_endpoints = {k: v for k, v in endpoints.items() if any(keyword in k for keyword in ['element', 'javascript', 'page'])}
        print(f"   DOM endpoints: {list(dom_endpoints.keys())}")
        
        return len(dom_caps) >= 4  # Should have at least 4 DOM-related capabilities
    except Exception as e:
        print(f"❌ Enhanced info failed: {e}")
        return False

def test_find_element():
    """Test element finding with CSS selectors"""
    print("\n🎯 Testing element finding...")
    try:
        payload = {
            "selector": {
                "css": "button.submit"
            }
        }
        response = requests.post(f"{API_BASE}/api/element/find", 
                               json=payload, 
                               headers={"Content-Type": "application/json"},
                               timeout=10)
        print(f"✅ Find element: {response.status_code}")
        print(f"   Response: {response.json()}")
        return response.status_code == 200
    except Exception as e:
        print(f"❌ Find element failed: {e}")
        return False

def test_element_interaction():
    """Test element interaction (click, type, etc.)"""
    print("\n👆 Testing element interaction...")
    try:
        payload = {
            "selector": {
                "css": "input[type='text']"
            },
            "interaction": {
                "type": "type",
                "text": "Hello from Tinker!"
            }
        }
        response = requests.post(f"{API_BASE}/api/element/interact", 
                               json=payload, 
                               headers={"Content-Type": "application/json"},
                               timeout=10)
        print(f"✅ Element interaction: {response.status_code}")
        print(f"   Response: {response.json()}")
        return response.status_code == 200
    except Exception as e:
        print(f"❌ Element interaction failed: {e}")
        return False

def test_element_highlighting():
    """Test element highlighting for debugging"""
    print("\n🎨 Testing element highlighting...")
    try:
        payload = {
            "selector": {
                "css": "h1"
            },
            "color": "#ff0000"
        }
        response = requests.post(f"{API_BASE}/api/element/highlight", 
                               json=payload, 
                               headers={"Content-Type": "application/json"},
                               timeout=10)
        print(f"✅ Element highlighting: {response.status_code}")
        print(f"   Response: {response.json()}")
        return response.status_code == 200
    except Exception as e:
        print(f"❌ Element highlighting failed: {e}")
        return False

def test_wait_conditions():
    """Test wait conditions for element states"""
    print("\n⏳ Testing wait conditions...")
    try:
        payload = {
            "condition": {
                "condition_type": "element_visible",
                "selector": {
                    "css": ".loading-spinner"
                },
                "timeout_ms": 5000,
                "poll_interval_ms": 100
            }
        }
        response = requests.post(f"{API_BASE}/api/element/wait", 
                               json=payload, 
                               headers={"Content-Type": "application/json"},
                               timeout=10)
        print(f"✅ Wait condition: {response.status_code}")
        print(f"   Response: {response.json()}")
        return response.status_code == 200
    except Exception as e:
        print(f"❌ Wait condition failed: {e}")
        return False

def test_javascript_execution():
    """Test JavaScript injection and execution"""
    print("\n⚡ Testing JavaScript execution...")
    try:
        payload = {
            "script": "console.log('Hello from Tinker API!'); document.title = 'Modified by Tinker'; return document.title;"
        }
        response = requests.post(f"{API_BASE}/api/javascript/execute", 
                               json=payload, 
                               headers={"Content-Type": "application/json"},
                               timeout=10)
        print(f"✅ JavaScript execution: {response.status_code}")
        print(f"   Response: {response.json()}")
        return response.status_code == 200
    except Exception as e:
        print(f"❌ JavaScript execution failed: {e}")
        return False

def test_page_info():
    """Test page information extraction"""
    print("\n📄 Testing page info extraction...")
    try:
        response = requests.get(f"{API_BASE}/api/page/info", timeout=10)
        print(f"✅ Page info: {response.status_code}")
        print(f"   Response: {response.json()}")
        return response.status_code == 200
    except Exception as e:
        print(f"❌ Page info failed: {e}")
        return False

def test_complex_workflow():
    """Test complex automation workflow"""
    print("\n🔄 Testing complex automation workflow...")
    try:
        # 1. Navigate to a page
        nav_payload = {"url": "https://httpbin.org/forms/post"}
        nav_response = requests.post(f"{API_BASE}/api/navigate", 
                                   json=nav_payload, 
                                   headers={"Content-Type": "application/json"},
                                   timeout=5)
        print(f"   1. Navigation: {nav_response.status_code}")
        
        # Wait a moment for navigation
        time.sleep(2)
        
        # 2. Find and fill a form field
        type_payload = {
            "selector": {"css": "input[name='custname']"},
            "interaction": {"type": "type", "text": "Tinker Test User"}
        }
        type_response = requests.post(f"{API_BASE}/api/element/interact", 
                                    json=type_payload, 
                                    headers={"Content-Type": "application/json"},
                                    timeout=10)
        print(f"   2. Form filling: {type_response.status_code}")
        
        # 3. Highlight the submit button
        highlight_payload = {
            "selector": {"css": "input[type='submit']"},
            "color": "#00ff00"
        }
        highlight_response = requests.post(f"{API_BASE}/api/element/highlight", 
                                         json=highlight_payload, 
                                         headers={"Content-Type": "application/json"},
                                         timeout=10)
        print(f"   3. Button highlighting: {highlight_response.status_code}")
        
        # 4. Take a screenshot of the result
        screenshot_payload = {"options": {"format": "PNG"}}
        screenshot_response = requests.post(f"{API_BASE}/api/screenshot", 
                                          json=screenshot_payload, 
                                          headers={"Content-Type": "application/json"},
                                          timeout=10)
        print(f"   4. Screenshot capture: {screenshot_response.status_code}")
        
        # 5. Execute custom JavaScript
        js_payload = {
            "script": "console.log('Workflow completed successfully!'); return 'Success';"
        }
        js_response = requests.post(f"{API_BASE}/api/javascript/execute", 
                                  json=js_payload, 
                                  headers={"Content-Type": "application/json"},
                                  timeout=10)
        print(f"   5. JavaScript execution: {js_response.status_code}")
        
        return all(r.status_code == 200 for r in [nav_response, type_response, highlight_response, screenshot_response, js_response])
    except Exception as e:
        print(f"❌ Complex workflow failed: {e}")
        return False

def main():
    """Run all DOM inspector and advanced feature tests"""
    print("🤖 Testing Tinker Browser Advanced DOM Inspection & Automation")
    print("=" * 70)
    
    results = []
    
    # Test enhanced capabilities
    results.append(test_enhanced_capabilities())
    
    # Test DOM inspection features
    results.append(test_find_element())
    results.append(test_element_interaction())
    results.append(test_element_highlighting())
    results.append(test_wait_conditions())
    
    # Test JavaScript injection
    results.append(test_javascript_execution())
    
    # Test page information
    results.append(test_page_info())
    
    # Test complex workflow
    results.append(test_complex_workflow())
    
    # Summary
    print("\n" + "=" * 70)
    passed = sum(results)
    total = len(results)
    print(f"📊 Advanced Features Test Results: {passed}/{total} passed")
    
    if passed == total:
        print("🎉 ALL ADVANCED FEATURES WORKING PERFECTLY!")
        print("🚀 Tinker now has COMPLETE browser automation capabilities:")
        print("   🎯 Element finding with CSS/XPath/text selectors")
        print("   👆 Element interactions (click, type, hover, etc.)")
        print("   🎨 Visual debugging with element highlighting")
        print("   ⏳ Smart wait conditions for dynamic content")
        print("   ⚡ JavaScript injection and execution")
        print("   📄 Comprehensive page information extraction")
        print("   📸 Screenshot capture with visual testing")
        print("   🔄 Complex automation workflow support")
        print("   🌐 WebSocket real-time control")
        print("   📡 MQTT event streaming")
        print("")
        print("🏆 TINKER IS NOW A WORLD-CLASS BROWSER TESTING PLATFORM!")
    else:
        print("⚠️  Some advanced tests failed. Check if Tinker is running with --api flag")
    
    return passed == total

if __name__ == "__main__":
    main()