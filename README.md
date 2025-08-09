# Secure File Upload System

A secure file upload system with JWT authentication using Keycloak, built with Rust backend and HTML/HTMX frontend.

## Architecture Overview

This project implements a **client-server architecture with an authentication layer** following these exact steps:

### 1. User Authentication
- Users must first authenticate with Keycloak before uploading files
- Keycloak issues an access token that proves the user's identity
- The access token is required to access the upload endpoint

### 2. Frontend Form
- Simple HTML form with HTMX upload extension
- Handles streaming multipart file uploads
- Provides real-time progress updates during upload

### 3. Backend Request
- HTMX sends HTTP POST request to `/upload` endpoint
- Includes file data (multipart upload) and access token in Authorization header
- Streams data directly to the Rust service

### 4. Authorization Check
- Rust backend validates access token against Keycloak
- Rejects requests with invalid or missing tokens
- Only authenticated users can upload files

### 5. File Processing
- Rust service streams multipart upload data
- Writes file directly to local `uploads/` directory
- Handles large files efficiently with streaming

### 6. Metadata Logging
- Creates JSON metadata entry with filename, user, timestamp, and size
- Appends entry to `uploads.json` file
- Maintains complete upload history

### 7. Progress Bar
- HTMX upload extension provides real-time progress updates
- Frontend displays dynamic progress bar during upload
- Shows upload percentage and file details

## Components

### Frontend (`/frontend/`)
- **Technology**: HTML + HTMX + JavaScript
- **Features**: 
  - Step-by-step authentication flow
  - Real-time upload progress tracking
  - Error handling and user feedback
  - Responsive design with modern UI

### Upload Proxy (`/upload-proxy/`)
- **Technology**: Rust + Actix-web framework
- **Features**:
  - JWT token validation against Keycloak JWKS
  - Streaming multipart file upload handling
  - Direct disk writing for efficiency
  - Comprehensive error handling and logging
  - Health check endpoint

### Authentication
- **Keycloak**: Identity and access management
- **PostgreSQL**: User data storage for Keycloak
- **JWT Tokens**: Secure authentication mechanism

### Storage
- **File Storage**: Local filesystem (`./uploads/`)
- **Metadata**: JSON file tracking (`uploads.json`)

## Project Structure

```
file-uploader/
├── frontend/
│   ├── index.html          # Main frontend interface
│   └── server.py           # Simple HTTP server for frontend
├── upload-proxy/
│   ├── src/main.rs         # Rust backend service
│   ├── Cargo.toml          # Rust dependencies
│   └── Dockerfile          # Container configuration
├── keycloak/               # Keycloak configuration
├── uploads/                # File storage directory
├── uploads.json            # Upload metadata log
├── docker-compose.yml      # Service orchestration
└── flow.md                 # Detailed flow documentation
```

## Key Features

### Security
- ✅ JWT-based authentication
- ✅ Token validation against Keycloak
- ✅ CORS protection
- ✅ Secure file handling

### Performance
- ✅ Streaming file uploads
- ✅ Direct disk writing
- ✅ Efficient multipart handling
- ✅ Real-time progress tracking

### User Experience
- ✅ Step-by-step authentication
- ✅ Visual progress indicators
- ✅ Error handling and feedback
- ✅ Modern responsive interface

## Quick Start

### Prerequisites
- Docker and Docker Compose
- Python 3 (for frontend server)
- Rust (for local development)

### Setup and Run

1. **Clone and Navigate**:
   ```bash
   cd file-uploader
   ```

2. **Start All Services**:
   ```bash
   docker-compose up --build
   ```

3. **Start Frontend Server** (in a separate terminal):
   ```bash
   cd frontend
   python3 server.py
   ```

4. **Access the Application**:
   - Frontend: http://localhost:8000
   - Keycloak Admin: http://localhost:8081 (admin/admin)
   - Upload API: http://localhost:3000

### Using the Application

1. **Access Frontend**: Navigate to http://localhost:8000
2. **Authenticate**: Click "Login with Keycloak"
   - Use test credentials: `testuser` / `testpass`
3. **Upload Files**: Select file and upload with real-time progress tracking
4. **View Results**: See upload confirmation with file details
5. **Check Metadata**: View `uploads.json` for upload history

### Test Credentials
- Username: `testuser`
- Password: `testpass`

## API Endpoints

### Upload Proxy (Port 3000)
- `GET /health` - Service health check
- `POST /upload` - File upload endpoint (requires JWT)

### Keycloak (Port 8080)
- Authentication and token management
- JWKS endpoint for token validation

### Frontend (Port 8000)
- Static file serving
- CORS-enabled for API calls

## Technologies Used

- **Backend**: Rust, Actix-web, Tokio
- **Frontend**: HTML, HTMX, JavaScript
- **Authentication**: Keycloak, JWT
- **Database**: PostgreSQL
- **Deployment**: Docker, Docker Compose
- **File Handling**: Multipart streaming, Direct I/O
