#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Test script for Tinker Browser Recording/Replay API

This script tests the new recording and playback functionality.
"""

import requests
import json
import time
import sys
import io

# Fix Windows console encoding
if sys.platform == 'win32':
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')

BASE_URL = "http://localhost:3003"

def test_api_health():
    """Test if API is running"""
    try:
        response = requests.get(f"{BASE_URL}/health", timeout=5)
        if response.status_code == 200:
            print("✓ API is healthy")
            return True
        else:
            print(f"✗ API health check failed: {response.status_code}")
            return False
    except requests.exceptions.RequestException as e:
        print(f"✗ Cannot connect to API: {e}")
        return False

def test_start_recording():
    """Test starting a recording"""
    print("\n=== Testing Recording Start ===")
    payload = {
        "name": "test_recording",
        "start_url": "https://example.com"
    }

    response = requests.post(
        f"{BASE_URL}/api/recording/start",
        json=payload,
        headers={"Content-Type": "application/json"}
    )

    print(f"Status: {response.status_code}")
    print(f"Response: {response.json()}")

    if response.status_code == 200 and response.json().get('success'):
        print("✓ Recording started successfully")
        return True
    else:
        print("✗ Failed to start recording")
        return False

def test_pause_recording():
    """Test pausing a recording"""
    print("\n=== Testing Recording Pause ===")

    response = requests.post(f"{BASE_URL}/api/recording/pause")

    print(f"Status: {response.status_code}")
    print(f"Response: {response.json()}")

    if response.status_code == 200 and response.json().get('success'):
        print("✓ Recording paused successfully")
        return True
    else:
        print("✗ Failed to pause recording")
        return False

def test_resume_recording():
    """Test resuming a recording"""
    print("\n=== Testing Recording Resume ===")

    response = requests.post(f"{BASE_URL}/api/recording/resume")

    print(f"Status: {response.status_code}")
    print(f"Response: {response.json()}")

    if response.status_code == 200 and response.json().get('success'):
        print("✓ Recording resumed successfully")
        return True
    else:
        print("✗ Failed to resume recording")
        return False

def test_stop_recording():
    """Test stopping a recording"""
    print("\n=== Testing Recording Stop ===")

    response = requests.post(f"{BASE_URL}/api/recording/stop")

    print(f"Status: {response.status_code}")
    print(f"Response: {response.json()}")

    if response.status_code == 200 and response.json().get('success'):
        print("✓ Recording stopped successfully")
        return True
    else:
        print("✗ Failed to stop recording")
        return False

def test_save_recording():
    """Test saving a recording"""
    print("\n=== Testing Recording Save ===")
    payload = {
        "path": "test_recording.json"
    }

    response = requests.post(
        f"{BASE_URL}/api/recording/save",
        json=payload,
        headers={"Content-Type": "application/json"}
    )

    print(f"Status: {response.status_code}")
    print(f"Response: {response.json()}")

    if response.status_code == 200 and response.json().get('success'):
        print("✓ Recording saved successfully")
        return True
    else:
        print("✗ Failed to save recording")
        return False

def test_load_recording():
    """Test loading a recording"""
    print("\n=== Testing Recording Load ===")
    payload = {
        "path": "test_recording.json"
    }

    response = requests.post(
        f"{BASE_URL}/api/recording/load",
        json=payload,
        headers={"Content-Type": "application/json"}
    )

    print(f"Status: {response.status_code}")
    print(f"Response: {response.json()}")

    if response.status_code == 200 and response.json().get('success'):
        print("✓ Recording loaded successfully")
        return True
    else:
        print("✗ Failed to load recording")
        return False

def test_playback_controls():
    """Test playback controls"""
    print("\n=== Testing Playback Controls ===")

    # Start playback
    print("\n1. Starting playback...")
    response = requests.post(f"{BASE_URL}/api/playback/start")
    print(f"   Status: {response.status_code}")
    print(f"   Response: {response.json()}")

    time.sleep(1)

    # Pause playback
    print("\n2. Pausing playback...")
    response = requests.post(f"{BASE_URL}/api/playback/pause")
    print(f"   Status: {response.status_code}")
    print(f"   Response: {response.json()}")

    time.sleep(1)

    # Resume playback
    print("\n3. Resuming playback...")
    response = requests.post(f"{BASE_URL}/api/playback/resume")
    print(f"   Status: {response.status_code}")
    print(f"   Response: {response.json()}")

    time.sleep(1)

    # Set speed
    print("\n4. Setting playback speed to 2x...")
    response = requests.post(
        f"{BASE_URL}/api/playback/speed",
        json={"speed": 2.0},
        headers={"Content-Type": "application/json"}
    )
    print(f"   Status: {response.status_code}")
    print(f"   Response: {response.json()}")

    time.sleep(1)

    # Stop playback
    print("\n5. Stopping playback...")
    response = requests.post(f"{BASE_URL}/api/playback/stop")
    print(f"   Status: {response.status_code}")
    print(f"   Response: {response.json()}")

    print("\n✓ Playback controls tested")
    return True

def test_playback_state():
    """Test getting playback state"""
    print("\n=== Testing Playback State ===")

    response = requests.get(f"{BASE_URL}/api/playback/state")

    print(f"Status: {response.status_code}")
    print(f"Response: {response.json()}")

    if response.status_code == 200:
        print("✓ Playback state retrieved successfully")
        return True
    else:
        print("✗ Failed to get playback state")
        return False

def main():
    """Run all tests"""
    print("=" * 60)
    print("Tinker Browser Recording/Replay API Test")
    print("=" * 60)

    # Check if API is running
    if not test_api_health():
        print("\n✗ API is not running. Please start the browser with:")
        print("  cargo run -- --api --url https://example.com")
        sys.exit(1)

    # Run tests
    tests = [
        ("Start Recording", test_start_recording),
        ("Pause Recording", test_pause_recording),
        ("Resume Recording", test_resume_recording),
        ("Stop Recording", test_stop_recording),
        ("Save Recording", test_save_recording),
        ("Load Recording", test_load_recording),
        ("Playback Controls", test_playback_controls),
        ("Playback State", test_playback_state),
    ]

    passed = 0
    failed = 0

    for name, test_func in tests:
        try:
            time.sleep(0.5)  # Brief pause between tests
            if test_func():
                passed += 1
            else:
                failed += 1
        except Exception as e:
            print(f"✗ Test '{name}' raised exception: {e}")
            failed += 1

    # Summary
    print("\n" + "=" * 60)
    print(f"Test Results: {passed} passed, {failed} failed")
    print("=" * 60)

    if failed == 0:
        print("\n🎉 All tests passed!")
        sys.exit(0)
    else:
        print(f"\n⚠️  {failed} test(s) failed")
        sys.exit(1)

if __name__ == "__main__":
    main()
