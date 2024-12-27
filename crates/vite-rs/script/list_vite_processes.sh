#!/bin/bash

# Tests if any vite dev servers are running, printing the PID and args of the process.
#
# This script was built for unix-like operation systems in mind.
# It likely won't work on Windows.

ps aux | grep -v grep | grep -v 'list_vite_processes.sh' | grep vite | tr -s " " | cut -d' ' -f2,11-
