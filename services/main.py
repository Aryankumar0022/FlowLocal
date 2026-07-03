import sys
import subprocess
import time
import signal
import os
from typing import List

# ============================================================
# services/main.py — Orchestrator for Python microservices
# Spawns Whisper, LLM, and RAG services.
# Used by the Tauri backend to launch all services at once.
# ============================================================

def main():
    print("[Orchestrator] Starting FlowLocal AI Services...")
    
    # Get the directory of this script to run modules correctly
    script_dir = os.path.dirname(os.path.abspath(__file__))
    
    # Define the services to run
    services = [
        {"name": "whisper", "module": "flowlocal_whisper.main", "dir": os.path.join(script_dir, "whisper")},
        {"name": "llm", "module": "flowlocal_llm.main", "dir": os.path.join(script_dir, "llm")},
        {"name": "rag", "module": "flowlocal_rag.main", "dir": os.path.join(script_dir, "rag")},
    ]
    
    processes: List[subprocess.Popen] = []
    
    def cleanup(signum=None, frame=None):
        print(f"\n[Orchestrator] Shutting down services...")
        for p in processes:
            if p.poll() is None:
                p.terminate()
        
        # Wait a bit, then kill
        time.sleep(1)
        for p in processes:
            if p.poll() is None:
                p.kill()
        
        print("[Orchestrator] All services stopped.")
        sys.exit(0)

    # Register signals
    signal.signal(signal.SIGINT, cleanup)
    signal.signal(signal.SIGTERM, cleanup)
    
    env = os.environ.copy()
    env["PYTHONPATH"] = os.pathsep.join([
        os.path.join(script_dir, "shared"),
        os.path.join(script_dir, "whisper"),
        os.path.join(script_dir, "llm"),
        os.path.join(script_dir, "rag")
    ])
    
    for svc in services:
        print(f"[Orchestrator] Spawning {svc['name']} (module: {svc['module']})")
        # Run using the same python executable that started this script
        p = subprocess.Popen(
            [sys.executable, "-m", svc['module']],
            cwd=svc['dir'],
            env=env
        )
        processes.append(p)
        
    print("[Orchestrator] All services spawned. Press Ctrl+C to stop.")
    
    # Monitor processes
    try:
        while True:
            for p in processes:
                if p.poll() is not None:
                    print(f"[Orchestrator] Error: A service exited unexpectedly (code {p.returncode}). Shutting down all.")
                    cleanup()
            time.sleep(1)
    except KeyboardInterrupt:
        cleanup()

if __name__ == "__main__":
    main()
