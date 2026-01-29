#!/usr/bin/env python3
"""Python client library for Mappy service with Unix socket and HTTP support."""

import os
import json
import socket
from typing import Optional, Dict, Any
from urllib.parse import quote

try:
    import httpx
    HTTPX_AVAILABLE = True
except ImportError:
    HTTPX_AVAILABLE = False


class MappyClient:
    """Client for Mappy service supporting Unix sockets and HTTP."""
    
    def __init__(
        self,
        socket_path: Optional[str] = None,
        http_url: Optional[str] = None,
    ):
        """
        Initialize Mappy client.
        
        Args:
            socket_path: Unix socket path (default: /var/run/reynard/mappy.sock)
            http_url: HTTP URL (e.g., http://localhost:8003)
        
        If both are provided, socket_path takes precedence.
        """
        if socket_path:
            self.socket_path = socket_path
            self.http_url = None
            self.use_socket = True
        elif http_url:
            self.socket_path = None
            self.http_url = http_url.rstrip('/')
            self.use_socket = False
        else:
            # Default to Unix socket
            self.socket_path = os.getenv(
                "MAPPY_SOCKET_PATH",
                "/var/run/reynard/mappy.sock"
            )
            self.http_url = None
            self.use_socket = True
    
    @classmethod
    def from_env(cls) -> "MappyClient":
        """Create client from environment variables."""
        socket_path = os.getenv("MAPPY_SOCKET_PATH")
        http_url = os.getenv("MAPPY_HTTP_URL")
        return cls(socket_path=socket_path, http_url=http_url)
    
    def _request_socket(self, method: str, path: str, body: Optional[bytes] = None) -> Dict[str, Any]:
        """Send HTTP request via Unix socket."""
        sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        try:
            sock.connect(self.socket_path)
            
            # Build HTTP request
            request = f"{method} {path} HTTP/1.1\r\n"
            request += "Host: localhost\r\n"
            request += "Content-Type: application/json\r\n"
            
            if body:
                request += f"Content-Length: {len(body)}\r\n"
            request += "\r\n"
            
            # Send request
            sock.sendall(request.encode())
            if body:
                sock.sendall(body)
            
            # Read response
            response = b""
            while True:
                chunk = sock.recv(4096)
                if not chunk:
                    break
                response += chunk
            
            # Parse HTTP response
            response_str = response.decode('utf-8')
            header_end = response_str.find("\r\n\r\n")
            if header_end == -1:
                raise ValueError("Invalid HTTP response")
            
            body_str = response_str[header_end + 4:]
            return json.loads(body_str)
        finally:
            sock.close()
    
    def _request_http(self, method: str, path: str, body: Optional[bytes] = None) -> Dict[str, Any]:
        """Send HTTP request via HTTP."""
        if not HTTPX_AVAILABLE:
            raise ImportError("httpx is required for HTTP requests. Install with: pip install httpx")
        
        url = f"{self.http_url}{path}"
        
        if method == "GET":
            response = httpx.get(url)
        elif method == "POST":
            response = httpx.post(url, content=body, headers={"Content-Type": "application/json"})
        else:
            raise ValueError(f"Unsupported method: {method}")
        
        response.raise_for_status()
        return response.json()
    
    def _request(self, method: str, path: str, body: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """Send request (auto-detects socket vs HTTP)."""
        body_bytes = json.dumps(body).encode() if body else None
        
        if self.use_socket:
            return self._request_socket(method, path, body_bytes)
        else:
            return self._request_http(method, path, body_bytes)
    
    def health(self) -> Dict[str, Any]:
        """Check server health."""
        return self._request("GET", "/health")
    
    def status(self) -> Dict[str, Any]:
        """Get server status."""
        return self._request("GET", "/status")
    
    def set(self, key: str, value: str) -> None:
        """Set a key-value pair."""
        self._request("POST", "/set", {"key": key, "value": value})
    
    def get(self, key: str) -> Optional[str]:
        """Get a value by key."""
        encoded_key = quote(key, safe='')
        response = self._request("GET", f"/get/{encoded_key}")
        return response.get("value")


# Convenience function
def create_client(socket_path: Optional[str] = None, http_url: Optional[str] = None) -> MappyClient:
    """Create a Mappy client."""
    return MappyClient(socket_path=socket_path, http_url=http_url)
