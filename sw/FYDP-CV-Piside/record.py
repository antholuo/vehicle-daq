#!/usr/bin/env python3
"""
Pi Camera Controller Server
Listens for commands from Windows client and controls camera capture.
"""

import os
import sys
import time
import json
import socket
import threading
import logging
from datetime import datetime
from pathlib import Path

# Try to import OpenCV for camera capture
try:
    import cv2
    CV2_AVAILABLE = True
except ImportError:
    CV2_AVAILABLE = False
    print("Warning: OpenCV not installed. Install with: pip install opencv-python")

# Configuration
CONFIG = {
    "host": "0.0.0.0",  # Listen on all interfaces
    "port": 8888,        # Command port (must match server.py)
    "image_interval": 0.1,  # Seconds between images when recording
    "base_image_dir": None,  # Will be set to ~/daq/logs/<date> at runtime
    "camera_timeout": 10,  # Camera detection timeout in seconds
    "max_cameras": 4,      # Maximum number of cameras to detect
    "image_quality": 100,   # JPEG quality (1-100)
    "image_format": ".jpg",  # Image file extension
}

# Global state
is_recording = False
active_cameras = []
camera_threads = []
stop_event = threading.Event()
session_timestamp = ""  # Store timestamp for current recording session

def setup_logging():
    """Set up logging to file and console"""
    log_path = "camera_server.log"
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(levelname)s - %(message)s',
        handlers=[
            logging.FileHandler(log_path),
            logging.StreamHandler(sys.stdout)
        ]
    )
    logging.info("=" * 50)
    logging.info("Pi Camera Controller Server Starting")
    logging.info("=" * 50)

def detect_cameras():
    """
    Detect available cameras by trying to open each camera index.
    Returns list of available camera indices.
    """
    if not CV2_AVAILABLE:
        logging.error("OpenCV not available. Cannot detect cameras.")
        return []
    
    available_cameras = []
    
    logging.info(f"Scanning for cameras (0 to {CONFIG['max_cameras']-1})...")
    
    for i in range(CONFIG['max_cameras']):
        try:
            cap = cv2.VideoCapture(i, cv2.CAP_V4L2)  # Use V4L2 for Linux
            if cap.isOpened():
                # Try to read a frame to confirm camera works
                ret, frame = cap.read()
                if ret:
                    # Get camera properties
                    width = int(cap.get(cv2.CAP_PROP_FRAME_WIDTH))
                    height = int(cap.get(cv2.CAP_PROP_FRAME_HEIGHT))
                    fps = cap.get(cv2.CAP_PROP_FPS)
                    
                    logging.info(f"Camera {i} detected: {width}x{height} @ {fps:.1f}fps")
                    available_cameras.append(i)
                    
                    # Test specific camera modes for Pi Camera if present
                    if i == 0:  # Often the Pi Camera module
                        # Try to set higher resolution for Pi Camera
                        cap.set(cv2.CAP_PROP_FRAME_WIDTH, 1920)
                        cap.set(cv2.CAP_PROP_FRAME_HEIGHT, 1080)
                        logging.info(f"  Set Pi Camera to 1920x1080 if available")
                else:
                    logging.warning(f"Camera {i} opened but cannot read frames")
                cap.release()
            else:
                logging.debug(f"No camera at index {i}")
        except Exception as e:
            logging.debug(f"Error checking camera {i}: {e}")
    
    logging.info(f"Found {len(available_cameras)} camera(s): {available_cameras}")
    return available_cameras

def create_session_directory(camera_indices):
    """
    Create session directory with timestamp and subdirectories for each camera.
    Returns dict with camera index -> directory path.
    """
    global session_timestamp
    
    # Generate timestamp for this session
    session_timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    
    # Get base directory: ~/daq/logs/<date>/
    home = Path.home()
    date_str = datetime.now().strftime("%Y-%m-%d")
    base_dir = home / "daq" / "logs" / date_str
    
    # Create base session directory: ~/daq/logs/<date>/<timestamp>/
    session_dir = base_dir / session_timestamp
    session_dir.mkdir(parents=True, exist_ok=True)
    
    # Create subdirectories for each camera
    camera_dirs = {}
    for cam_idx in camera_indices:
        cam_dir = session_dir / f"camera_{cam_idx}"
        cam_dir.mkdir(exist_ok=True)
        camera_dirs[cam_idx] = str(cam_dir)
        logging.info(f"Camera {cam_idx} directory: {cam_dir}")
    
    # Create a session info file
    session_info = {
        "session_timestamp": session_timestamp,
        "start_time": datetime.now().isoformat(),
        "cameras": camera_indices,
        "image_interval": CONFIG['image_interval'],
        "image_quality": CONFIG['image_quality'],
        "image_format": CONFIG['image_format']
    }
    
    info_file = session_dir / "session_info.json"
    with open(info_file, 'w') as f:
        json.dump(session_info, f, indent=2)
    
    logging.info(f"Created session directory: {session_dir}")
    return camera_dirs

def camera_capture_loop(camera_index, output_dir, stop_flag):
    """
    Main capture loop for a single camera.
    Runs in its own thread.
    """
    global is_recording, session_timestamp
    
    if not CV2_AVAILABLE:
        logging.error(f"OpenCV not available. Camera {camera_index} cannot capture.")
        return
    
    logging.info(f"Starting capture loop for camera {camera_index} in session {session_timestamp}")
    
    # Initialize camera
    cap = None
    try:
        # Use V4L2 backend for Linux/Raspberry Pi
        cap = cv2.VideoCapture(camera_index, cv2.CAP_V4L2)
        
        if not cap.isOpened():
            logging.error(f"Failed to open camera {camera_index}")
            return
        
        # Configure camera settings
        # Try to set reasonable defaults
        cap.set(cv2.CAP_PROP_FRAME_WIDTH, 1920)  # Try 1080p
        cap.set(cv2.CAP_PROP_FRAME_HEIGHT, 1080)
        cap.set(cv2.CAP_PROP_FPS, 30)
        cap.set(cv2.CAP_PROP_BUFFERSIZE, 1)  # Reduce latency
        
        frame_count = 0
        
        while not stop_flag.is_set() and is_recording:
            try:
                # Capture frame
                ret, frame = cap.read()
                
                if not ret:
                    logging.warning(f"Camera {camera_index}: Failed to capture frame")
                    time.sleep(0.1)
                    continue
                
                # Generate filename with timestamp and frame count
                frame_timestamp = datetime.now().strftime("%H%M%S_%f")[:-3]
                filename = f"cam{camera_index}_{frame_timestamp}_{frame_count:06d}{CONFIG['image_format']}"
                filepath = os.path.join(output_dir, filename)
                
                # Save image
                cv2.imwrite(
                    filepath, 
                    frame, 
                    [cv2.IMWRITE_JPEG_QUALITY, CONFIG['image_quality']]
                )
                
                frame_count += 1
                
                # Log periodically
                if frame_count % 10 == 0:
                    logging.info(f"Camera {camera_index}: Saved {frame_count} images to session {session_timestamp}")
                
                # Wait for next capture
                time.sleep(CONFIG['image_interval'])
                
            except Exception as e:
                logging.error(f"Camera {camera_index} capture error: {e}")
                time.sleep(1)
        
        logging.info(f"Stopping capture loop for camera {camera_index} (session: {session_timestamp})")
        
    except Exception as e:
        logging.error(f"Camera {camera_index} initialization error: {e}")
    finally:
        if cap is not None:
            cap.release()
            logging.info(f"Camera {camera_index} released")

def start_recording(camera_indices):
    """
    Start recording from all detected cameras.
    Each camera runs in its own thread.
    """
    global is_recording, active_cameras, camera_threads, stop_event, session_timestamp
    
    if is_recording:
        logging.warning("Recording already in progress")
        return False
    
    if not camera_indices:
        logging.error("No cameras available to record")
        return False
    
    logging.info(f"Starting recording from cameras: {camera_indices}")
    
    # Create session directory with timestamp
    camera_dirs = create_session_directory(camera_indices)
    
    # Reset stop event
    stop_event.clear()
    
    # Set recording flag
    is_recording = True
    active_cameras = camera_indices.copy()
    
    # Start a thread for each camera
    camera_threads = []
    for cam_idx in camera_indices:
        thread = threading.Thread(
            target=camera_capture_loop,
            args=(cam_idx, camera_dirs[cam_idx], stop_event),
            daemon=True
        )
        thread.start()
        camera_threads.append(thread)
        logging.info(f"Started thread for camera {cam_idx} in session {session_timestamp}")
    
    logging.info(f"Recording started with {len(camera_indices)} camera(s) in session {session_timestamp}")
    return True

def stop_recording():
    """
    Stop recording from all cameras.
    """
    global is_recording, active_cameras, camera_threads, session_timestamp
    
    if not is_recording:
        logging.warning("No recording in progress")
        return True
    
    logging.info(f"Stopping recording (session: {session_timestamp})...")
    
    # Set flags to stop
    is_recording = False
    stop_event.set()
    
    # Wait for all threads to finish (with timeout)
    for thread in camera_threads:
        thread.join(timeout=5.0)
    
    # Update session info with end time
    try:
        home = Path.home()
        date_str = datetime.now().strftime("%Y-%m-%d")
        base_dir = home / "daq" / "logs" / date_str
        session_dir = base_dir / session_timestamp
        info_file = session_dir / "session_info.json"
        if info_file.exists():
            with open(info_file, 'r') as f:
                session_info = json.load(f)
            session_info['end_time'] = datetime.now().isoformat()
            with open(info_file, 'w') as f:
                json.dump(session_info, f, indent=2)
            logging.info(f"Updated session info for {session_timestamp}")
    except Exception as e:
        logging.error(f"Error updating session info: {e}")
    
    # Clear lists and reset session timestamp
    active_cameras = []
    camera_threads = []
    session_timestamp = ""
    
    logging.info("Recording stopped")
    return True

def handle_client_command(command):
    """
    Handle incoming commands from client.
    Returns response message.
    """
    global session_timestamp
    command = command.strip().lower()
    
    if command == "start":
        if is_recording:
            return f"ERROR: Already recording (session: {session_timestamp})"
        
        # Detect cameras
        cameras = detect_cameras()
        if not cameras:
            return "ERROR: No cameras detected"
        
        # Start recording
        if start_recording(cameras):
            return f"OK: Recording started with {len(cameras)} camera(s) in session {session_timestamp}"
        else:
            return "ERROR: Failed to start recording"
    
    elif command == "stop":
        if not is_recording:
            return "OK: No recording in progress"
        
        if stop_recording():
            return f"OK: Recording stopped (session: {session_timestamp})"
        else:
            return "ERROR: Failed to stop recording"
    
    elif command == "status":
        status = "recording" if is_recording else "idle"
        cameras = active_cameras if is_recording else []
        if is_recording:
            return f"STATUS: {status}, Session: {session_timestamp}, Cameras: {cameras}"
        else:
            return f"STATUS: {status}, Cameras: {cameras}"
    
    elif command == "detect":
        cameras = detect_cameras()
        return f"DETECT: Found {len(cameras)} camera(s): {cameras}"
    
    elif command == "help":
        return "COMMANDS: start, stop, status, detect, help"
    
    else:
        return f"ERROR: Unknown command '{command}'"

def command_server():
    """
    Main command server that listens for incoming commands.
    Runs in its own thread.
    """
    server_socket = None
    
    try:
        # Create TCP socket
        server_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        server_socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        server_socket.settimeout(1.0)  # Timeout for accept
        
        # Bind to port
        server_socket.bind((CONFIG['host'], CONFIG['port']))
        server_socket.listen(5)
        
        logging.info(f"Command server listening on {CONFIG['host']}:{CONFIG['port']}")
        logging.info(f"Available commands: start, stop, status, detect, help")
        
        while True:
            try:
                # Accept incoming connection
                client_socket, client_address = server_socket.accept()
                logging.info(f"Connection from {client_address}")
                
                try:
                    # Receive command
                    data = client_socket.recv(1024).decode('utf-8').strip()
                    if data:
                        logging.info(f"Received command: {data}")
                        
                        # Handle command
                        response = handle_client_command(data)
                        
                        # Send response
                        client_socket.sendall(response.encode('utf-8'))
                        logging.info(f"Sent response: {response}")
                    
                except Exception as e:
                    logging.error(f"Error handling client {client_address}: {e}")
                finally:
                    client_socket.close()
                    
            except socket.timeout:
                # Timeout is expected, just continue
                continue
            except KeyboardInterrupt:
                break
            except Exception as e:
                logging.error(f"Server error: {e}")
                time.sleep(1)
                
    except Exception as e:
        logging.error(f"Failed to start server: {e}")
    finally:
        if server_socket:
            server_socket.close()
        logging.info("Command server stopped")

def cleanup():
    """
    Cleanup function to ensure resources are released.
    """
    global is_recording
    
    logging.info("Cleaning up...")
    
    # Stop recording if active
    if is_recording:
        stop_recording()
    
    # Additional cleanup if needed
    logging.info("Cleanup complete")

def main():
    """
    Main entry point.
    """
    setup_logging()
    
    # Check OpenCV availability
    if not CV2_AVAILABLE:
        logging.warning("""
        OpenCV is not installed. Camera functionality will not work.
        Install with: sudo apt-get install python3-opencv
        Or: pip3 install opencv-python
        """)
    
    # Base directory will be created dynamically in create_session_directory()
    # as ~/daq/logs/<date>/ to match the Rust data logger location
    
    # Initial camera detection
    cameras = detect_cameras()
    logging.info(f"Initial camera detection: {len(cameras)} camera(s) found")
    
    # Start command server in background thread
    server_thread = threading.Thread(target=command_server, daemon=True)
    server_thread.start()
    
    try:
        logging.info("Server is running. Press Ctrl+C to stop.")
        
        # Keep main thread alive
        while True:
            # Periodic status update (optional)
            if is_recording:
                # Check if any camera threads died
                for i, thread in enumerate(camera_threads):
                    if not thread.is_alive() and i < len(active_cameras):
                        logging.error(f"Camera {active_cameras[i]} thread died unexpectedly in session {session_timestamp}")
            
            time.sleep(5)
            
    except KeyboardInterrupt:
        logging.info("\nShutting down...")
    except Exception as e:
        logging.error(f"Unexpected error: {e}")
    finally:
        cleanup()
        logging.info("Server stopped.")

if __name__ == "__main__":
    main()
