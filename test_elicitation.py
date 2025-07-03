#!/usr/bin/env python3
"""
Script to simulate an MCP client with elicitation support
"""

import json
import subprocess
import sys
import time

def send_mcp_request(process, request):
    """Send JSON-RPC request to MCP server"""
    request_json = json.dumps(request)
    print(f"→ Sending: {request_json}")
    process.stdin.write(request_json + '\n')
    process.stdin.flush()

def read_mcp_response(process):
    """Read response from MCP server"""
    try:
        line = process.stdout.readline()
        if line:
            response = json.loads(line.strip())
            print(f"← Received: {json.dumps(response, ensure_ascii=False, indent=2)}")
            return response
    except Exception as e:
        print(f"Error reading response: {e}")
    return None

def main():
    # Start MCP server
    print("🚀 Starting MCP server...")
    process = subprocess.Popen(
        ['./target/debug/vkteams-bot-mcp'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=0
    )

    try:
        # 1. Initialization
        print("\n📞 1. Initializing server...")
        init_request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-06-18",
                "capabilities": {
                    "elicitation": {}  # Declare elicitation support!
                },
                "clientInfo": {
                    "name": "ElicitationTestClient",
                    "version": "1.0.0"
                }
            }
        }

        send_mcp_request(process, init_request)
        init_response = read_mcp_response(process)

        if not init_response:
            print("❌ No initialization response received")
            return

        # 2. Initialization notification
        print("\n✅ 2. Sending initialization notification...")
        initialized_notification = {
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        }

        send_mcp_request(process, initialized_notification)

        # 3. Call send_text without chat_id (should trigger elicitation)
        print("\n📨 3. Sending message without chat_id...")
        send_text_request = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "send_text",
                "arguments": {
                    "text": "Hello! This is a test message via elicitation! 🎯"
                }
            }
        }

        send_mcp_request(process, send_text_request)

        # 4. Wait for elicitation request
        print("\n⏳ 4. Waiting for elicitation request...")
        time.sleep(0.1)  # Small pause

        elicitation_request = read_mcp_response(process)

        if elicitation_request and elicitation_request.get("method") == "elicitation/create":
            print("🎯 Received elicitation request!")

            # 5. Respond to elicitation with chat_id
            print("\n💬 5. Responding to elicitation with chat_id...")
            elicitation_response = {
                "jsonrpc": "2.0",
                "id": elicitation_request["id"],
                "result": {
                    "action": "accept",
                    "content": {
                        "chat_id": "1111111@chat.agent"  # Provide chat_id
                    }
                }
            }

            send_mcp_request(process, elicitation_response)

            # 6. Wait for final response
            print("\n🏁 6. Waiting for final response...")
            final_response = read_mcp_response(process)

            if final_response:
                print("✅ Received final response!")
                print("🎉 Elicitation flow completed successfully!")
            else:
                print("❌ No final response received")
        else:
            print("❌ No elicitation request received")
            print("📋 Received instead:", elicitation_request)

    except KeyboardInterrupt:
        print("\n⏹️  Interrupted by user")
    except Exception as e:
        print(f"❌ Error: {e}")
    finally:
        print("\n🛑 Terminating process...")
        process.terminate()
        process.wait()

if __name__ == "__main__":
    main()
