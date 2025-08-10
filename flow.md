# File Upload System - Assignment Requirements & Flow Documentation

## 📋 Assignment Overview

**Project Goal**: Build a secure file upload system using modern web technologies where authenticated users can upload files to a local directory with metadata logging.

**Required Technologies**:
- **HTMX**: Modern JavaScript library for dynamic web interfaces
- **Rust**: Systems programming language for the backend service
- **OAuth2**: Standard protocol for secure authentication
- **Keycloak**: Identity and access management server (running in Docker)

## 🔍 Understanding the Assignment Terms

### What is HTMX?
HTMX is a modern JavaScript library that allows you to create dynamic web pages without writing complex JavaScript. It lets you:
- Send requests to servers without page reloads
- Update parts of the page dynamically
- Handle file uploads with progress bars
- Create interactive forms easily

### What is Rust?
Rust is a systems programming language that's:
- **Fast**: As fast as C/C++
- **Safe**: Prevents common programming errors
- **Modern**: Has excellent tooling and package management
- **Perfect for web services**: Great for building APIs

### What is OAuth2?
OAuth2 is a security standard that:
- Allows users to log in without sharing passwords
- Uses tokens instead of passwords for authentication
- Is widely used by Google, Facebook, GitHub, etc.
- Provides secure access to protected resources

### What is Keycloak?
Keycloak is an open-source identity management server that:
- Handles user authentication and authorization
- Supports OAuth2 and OpenID Connect
- Manages user accounts and passwords
- Issues secure JWT tokens

## 🏗️ System Architecture

```mermaid
graph TB
    subgraph "Frontend (HTMX)"
        UI[👤 User Interface]
        Form[📝 Upload Form]
        Progress[📊 Progress Bar]
    end
    
    subgraph "Authentication (Keycloak)"
        Login[🔐 Login Page]
        Token[🎫 JWT Token]
        DB[(🗄️ User Database)]
    end
    
    subgraph "Backend (Rust)"
        API[🚀 Upload API]
        Auth[🔍 Token Validation]
        File[📁 File Handler]
        Meta[📋 Metadata Logger]
    end
    
    subgraph "Storage"
        Files[📂 Uploads Directory]
        Log[📄 uploads.json]
    end
    
    %% User Flow
    UI --> Login
    Login --> Token
    Token --> Form
    
    %% Upload Flow
    Form --> API
    API --> Auth
    Auth --> File
    File --> Files
    File --> Meta
    Meta --> Log
    
    %% Progress Updates
    File --> Progress
    
    %% Database
    Login --> DB
    
    %% Styling
    classDef frontend fill:#e3f2fd
    classDef auth fill:#f3e5f5
    classDef backend fill:#e8f5e8
    classDef storage fill:#fff3e0
    
    class UI,Form,Progress frontend
    class Login,Token,DB auth
    class API,Auth,File,Meta backend
    class Files,Log storage
```

## 🔄 Complete User Flow

```mermaid
sequenceDiagram
    participant U as 👤 User
    participant F as 🌐 Frontend (HTMX)
    participant K as 🔐 Keycloak
    participant R as 🦀 Rust Backend
    participant S as 💾 Storage
    
    Note over U,S: Step 1: Authentication
    U->>F: Access upload page
    F->>K: Redirect to login
    K->>U: Show login form
    U->>K: Enter credentials
    K->>F: Return with access token
    F->>F: Store token in session
    
    Note over U,S: Step 2: File Upload
    U->>F: Select file & click upload
    F->>R: POST /upload + file + token
    
    Note over U,S: Step 3: Authorization
    R->>K: Validate JWT token
    K->>R: Token is valid (user info)
    
    Note over U,S: Step 4: File Processing
    R->>S: Save file to uploads/
    R->>S: Log metadata to uploads.json
    R->>F: Return success response
    
    Note over U,S: Step 5: User Feedback
    F->>U: Show progress bar & success
```

## 🎯 Assignment Requirements Breakdown

### 1. **User Authentication** ✅
```mermaid
graph LR
    A[User visits site] --> B[Keycloak login]
    B --> C[Enter username/password]
    C --> D[Keycloak issues JWT token]
    D --> E[Token stored in browser]
    
    style A fill:#e1f5fe
    style D fill:#c8e6c9
    style E fill:#fff3e0
```

**What happens**: Before uploading, users must log in through Keycloak, which gives them a special token (JWT) that proves they're authenticated.

### 2. **Frontend Form** ✅
```mermaid
graph LR
    A[HTML Form] --> B[HTMX Upload Extension]
    B --> C[File Selection]
    C --> D[Progress Bar]
    D --> E[Submit to Backend]
    
    style A fill:#e3f2fd
    style B fill:#f3e5f5
    style D fill:#e8f5e8
```

**What happens**: Simple HTML form uses HTMX to handle file uploads with real-time progress updates.

### 3. **Backend Request** ✅
```mermaid
graph LR
    A[HTMX Form] --> B[POST /upload]
    B --> C[Multipart Data]
    C --> D[Authorization Header]
    D --> E[Rust Service]
    
    style A fill:#e3f2fd
    style C fill:#fff3e0
    style D fill:#f3e5f5
    style E fill:#e8f5e8
```

**What happens**: HTMX sends the file data and user's authentication token to the Rust backend service.

### 4. **Authorization Check** ✅
```mermaid
graph LR
    A[Receive Request] --> B[Extract JWT Token]
    B --> C[Validate with Keycloak]
    C --> D{Token Valid?}
    D -->|Yes| E[Allow Upload]
    D -->|No| F[Reject Request]
    
    style A fill:#e8f5e8
    style C fill:#f3e5f5
    style E fill:#c8e6c9
    style F fill:#ffcdd2
```

**What happens**: Rust service checks if the user's token is valid by asking Keycloak to verify it.

### 5. **File Processing** ✅
```mermaid
graph LR
    A[Valid Token] --> B[Stream File Data]
    B --> C[Write to Disk]
    C --> D[Save to uploads/]
    
    style A fill:#e8f5e8
    style B fill:#fff3e0
    style D fill:#c8e6c9
```

**What happens**: Once authorized, the Rust service streams the file data and saves it to the local `uploads/` directory.

### 6. **Metadata Logging** ✅
```mermaid
graph LR
    A[File Saved] --> B[Create Metadata]
    B --> C[filename, user, timestamp]
    C --> D[Append to uploads.json]
    
    style A fill:#e8f5e8
    style B fill:#fff3e0
    style D fill:#c8e6c9
```

**What happens**: After successful upload, the system logs metadata (filename, user, timestamp) to `uploads.json`.

### 7. **Progress Bar** ✅
```mermaid
graph LR
    A[Upload Starts] --> B[HTMX Progress Events]
    B --> C[Update Progress Bar]
    C --> D[Show Percentage]
    D --> E[Upload Complete]
    
    style A fill:#e3f2fd
    style B fill:#f3e5f5
    style C fill:#e8f5e8
    style D fill:#fff3e0
```

**What happens**: HTMX provides real-time progress updates that show the user how much of their file has been uploaded.

## 🐳 Docker Compose Services

```mermaid
graph TB
    subgraph "Docker Compose Services"
        subgraph "Authentication Layer"
            K[🔐 Keycloak<br/>Port 8081]
            DB[(🗄️ PostgreSQL<br/>Database)]
        end
        
        subgraph "Backend Layer"
            R[🦀 Upload Proxy<br/>Port 3000<br/>Rust Service]
        end
        
        subgraph "Storage Layer"
            V1[📁 uploads/ Volume]
            V2[📄 uploads.json Volume]
        end
    end
    
    K --> DB
    R --> K
    R --> V1
    R --> V2
    
    classDef auth fill:#f3e5f5
    classDef backend fill:#e8f5e8
    classDef storage fill:#fff3e0
    
    class K,DB auth
    class R backend
    class V1,V2 storage
```

## 📁 Project Structure

```
file-uploader/
├── 📄 docker-compose.yml          # 🐳 Service orchestration
├── 📁 frontend/                   # 🌐 HTMX frontend
│   ├── 📄 index.html             # 🔐 Login page
│   ├── 📄 upload.html            # 📝 Upload interface
│   ├── 📄 config.js              # ⚙️ Configuration
│   └── 📄 server.py              # 🐍 Development server
├── 📁 upload-proxy/              # 🦀 Rust backend
│   ├── 📄 Cargo.toml            # 📦 Dependencies
│   ├── 📄 Dockerfile            # 🐳 Container build
│   └── 📁 src/
│       ├── 📄 main.rs           # 🚀 Service entry point
│       ├── 📄 auth.rs           # 🔍 Token validation
│       ├── 📄 handlers.rs       # 📝 Request handlers
│       └── 📄 metadata.rs       # 📋 Metadata logging
├── 📁 keycloak/                  # 🔐 Authentication
│   └── 📄 realm-export.json     # ⚙️ Keycloak config
├── 📁 uploads/                   # 📂 File storage
├── 📄 uploads.json              # 📋 Upload metadata
└── 📄 README.md                 # 📖 Documentation
```

## 🚀 How to Run the Assignment

### 1. Start All Services
```bash
# Start Keycloak, PostgreSQL, and Rust backend
docker-compose up --build
```

### 2. Start Frontend Server
```bash
# In a new terminal
cd frontend
python3 server.py
```

### 3. Access the Application
- **Frontend**: http://localhost:8000
- **Keycloak Admin**: http://localhost:8081 (admin/admin)
- **Upload API**: http://localhost:3000

### 4. Test the System
1. Go to http://localhost:8000
2. Click "Login with Keycloak"
3. Use test credentials: `testuser` / `testpass`
4. Select a file and upload
5. Watch the progress bar and success message

## 🔧 Technical Implementation Details

### Frontend (HTMX)
- **File**: `frontend/upload.html`
- **Features**: 
  - HTMX upload extension for file handling
  - Real-time progress bars
  - JWT token management
  - Modern responsive design

### Backend (Rust)
- **Framework**: Actix-web
- **Features**:
  - JWT token validation
  - Streaming file uploads
  - Multipart data handling
  - Metadata logging

### Authentication (Keycloak)
- **Protocol**: OAuth2 with PKCE
- **Database**: PostgreSQL
- **Features**:
  - User management
  - JWT token issuance
  - Public key endpoint (JWKS)

## 📊 Data Flow Summary

1. **User Authentication** → Keycloak issues JWT token
2. **File Selection** → HTMX form with progress tracking
3. **Upload Request** → Rust service with token validation
4. **File Processing** → Stream to disk in uploads/
5. **Metadata Logging** → Append to uploads.json
6. **User Feedback** → Progress bar and success message

## ✅ Assignment Compliance Checklist

- [x] **HTMX**: Used for frontend with upload extension
- [x] **Rust**: Backend service with Actix-web
- [x] **OAuth2**: PKCE flow with Keycloak
- [x] **Keycloak (Docker)**: Containerized authentication
- [x] **User Authentication**: Required before upload
- [x] **File Upload**: Multipart streaming to disk
- [x] **Metadata Logging**: JSON format with filename, user, timestamp
- [x] **Progress Bar**: Real-time upload progress
- [x] **Docker Compose**: All services orchestrated

**🎉 This implementation fully satisfies all assignment requirements!**
