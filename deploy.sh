#!/bin/bash

# Function to check for necessary dependencies
check_dependencies() {
  local dependencies=("docker" "docker-compose")

  for dep in "${dependencies[@]}"; do
    if ! command -v $dep &> /dev/null; then
      echo "$dep is not installed. Please install it before running this script."
      exit 1
    fi
  done
}

# Function to check if the script is run on Linux or macOS
check_os() {
  if [[ "$OSTYPE" != "linux-gnu"* && "$OSTYPE" != "darwin"* ]]; then
    echo "This script can only be run on Linux or macOS."
    exit 1
  fi
}

# Function to ask for sudo elevation if not run as sudo
check_sudo() {
  if [[ $EUID -ne 0 ]]; then
    echo "This script requires sudo privileges. Please run with sudo."
    exit 1
  fi
}

# Function to clean up Docker images and containers
cleanup_docker() {
  echo "Cleaning up Docker images and containers..."
  docker system prune -af
}

# Function to tear down, rebuild, and run the application
run_application() {
  echo "Stopping and removing any existing Docker containers..."
  docker-compose down

  echo "Building and running the Docker containers..."
  docker-compose up --build -d

  echo "Fetching logs of the Docker container..."
  docker-compose logs photo-album
}

# Check if the script is run on Linux or macOS
check_os

# Check for dependencies
check_dependencies

# Check for sudo privileges
check_sudo

# Clean up Docker
cleanup_docker

# Run the application
run_application

echo "Script execution completed."
