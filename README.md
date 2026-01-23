# MySQL GUI Administration Tool
Developed by okcoe21

A high-performance, modern, and secure MySQL administration interface designed for local development and professional database management. This is an open-source project.

---

## Overview

This tool provides a streamlined web-based interface for interacting with MySQL databases without the overhead of heavy administration suites. Built with Next.js and TypeScript, it focuses on visual excellence, security, and a robust user experience for developers.

---

## Key Features

### Database & Server Management
- **Server Overview**: Centrally manage all databases on your MySQL server from a single view.
- **Database Operations**: Perform global actions like creating new databases or dropping existing ones with built-in safety confirmations.
- **Auto-Sync Navigation**: Sidebar automatically refreshes to reflect any changes made to the database structure.

### Advanced Schema & Data Tools
- **Professional Table Designer**: Define complex schemas with multiple columns, specific data types, lengths, and constraint flags (Primary Key, Auto-Increment, Nullability).
- **Data Exploration**: High-speed browsing of table data with a clean, formatted grid layout.
- **Advanced SQL Editor**: Execute custom SQL queries with a powerful editor featuring query formatting and one-click clear options.

### Migration & Portability
- **Secure Export**: Generate complete SQL dumps including both table structures and data for easy backups.
- **Reliable Import**: Process and execute large SQL scripts with detailed execution reporting and error tracking.

---

## Security Architecture

The application is engineered with a security-first mindset:

- **Local Data Privacy**: Designed for local execution to ensure that sensitive database credentials never leave your environment.
- **Encrypted Sessions**: Utilizes JWT-based session management with server-side encryption to store MySQL credentials.
- **Sanitization Layer**: Employs strict identifier sanitization and backtick-wrapping for all database, table, and column names to mitigate SQL injection risks.
- **Destructive Action Safeguards**: Real-time analysis of SQL queries to detect and require confirmation for DROP, TRUNCATE, and DELETE operations.

---

## Technical Stack

- **Frontend**: Next.js 15, React 19, Lucide React (Icons)
- **Backend**: Next.js API Routes (Node.js runtime)
- **Database Driver**: mysql2 (with Connection Pooling)
- **Styling**: Unified CSS Modules for a premium, custom interface
- **Security**: jose (JWT), Encrypted Credentials

---

## Installation and Setup

### Prerequisites

- Node.js v18.0.0 or higher
- A running MySQL Server instance

### Steps to Run Locally

1. **Clone the project**:
   ```bash
   git clone https://github.com/okcoe21/MYSQL-GUI.git
   cd mysql-gui
   ```

2. **Install dependencies**:
   ```bash
   npm install
   ```

3. **Configure Environment**:
   Create a `.env.local` file in the root directory. You can generate a secure secret using:
   ```bash
   # Linux/macOS
   echo "SESSION_SECRET=$(openssl rand -hex 32)" > .env.local
   ```

4. **Launch Application**:
   ```bash
   npm run dev
   ```
   Access the interface at http://localhost:3000.

---

## Contributing

As an open-source project, contributions are welcome. Please feel free to submit issues or pull requests to help improve the tool for the developer community.

---

## License

This project is open-source. Please refer to the LICENSE file for more information (MIT License recommended).

---
*Made by okcoe21*
