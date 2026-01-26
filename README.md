# 🗄️ MySQL GUI Administration Tool
[![Next.js](https://img.shields.io/badge/Next.js-15-black?style=flat-square&logo=next.js)](https://nextjs.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5-blue?style=flat-square&logo=typescript)](https://www.typescriptlang.org/)
[![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)](LICENSE)

A high-performance, modern, and secure MySQL administration interface designed for local development and professional database management. This tool bridges the gap between complex enterprise suites and simple command-line tools with a **vibrant, dark-first UI** and **real-time observability**.

---

## Key Features

### Server & Schema Management
- **Real-time Monitoring**: Live dashboard showing Queries/sec, Active Connections, and Network Traffic via SVG-based time-series charts.
- **Advanced Object Support**: Full management suite for **Tables, Views, Stored Procedures, and Functions**.
- **User Permissions**: Audit and manage MySQL users, hosts, and account lock statuses centrally.
- **Slow Query Forensic**: Identify execution bottlenecks by analyzing the server's slow query log directly.

### Developer Productivity
- **Visual ER Diagrams**: Auto-generated interactive relationship maps to visualize your database architecture.
- **Intelligent Query Builder**: Visual "No-Code" interface for generating optimized `SELECT` queries.
- **Local History & Favorites**: Persisted query snippets and execution history for rapid workflow.
- **Smart Seeding**: Built-in **Mock Data Generator** for names, emails, dates, and random text with bulk insertion support.

### Data Manipulation & Portability
- **Direct Cell Editing**: Double-click any cell in the data grid to update the record instantly with server-side validation.
- **High-Performance Grid**: Server-side pagination, sorting, and filtering optimized for tables with millions of rows.
- **Multi-Format Portability**: Export your entire database or individual tables to **SQL, JSON, or CSV**.
- **Robust Importer**: Execute large SQL scripts with detailed statement-by-statement progress and error tracking (with rollback awareness).

---

## Security Architecture

The application is engineered with a **Security-First** mindset:

- **Local-Only Boundary**: Credentials and sensitive database data never leave your local environment; there is no secondary cloud storage.
- **Encrypted Session Layer**: Uses JWT (JSON Web Tokens) with a server-side `SESSION_SECRET` to encrypt MySQL connection details.
- **Advanced Injection Mitigation**: Uses a strict sanitization layer for all identifiers and utilizes parameterization for values.
- **Safety Confirmations**: Real-time analysis of SQL payloads detects destructive commands (`DROP`, `DELETE`, `TRUNCATE`) and requires user confirmation.

---

## Technical Stack

- **Framework**: [Next.js 15](https://nextjs.org/) (App Router, Node.js Runtime)
- **UI Engine**: React 19 + Lucide Icons
- **Typography & Styling**: Unified CSS Design System (Vanilla CSS Modules)
- **Database Engine**: `mysql2` with optimized Connection Pooling
- **Authentication**: `jose` for encrypted session management

---

## Installation & Setup

### Prerequisites
- **Node.js**: v18.0.0 or higher
- **MySQL**: A running instance (local or remote)

### Quick Start
1. **Clone & Install**:
   ```bash
   git clone https://github.com/okcoe21/MYSQL-GUI.git
   cd MYSQL-GUI
   npm install
   ```

2. **Environment Configuration**:
   Create a `.env.local` file. Generate a secure secret with:
   ```bash
   # Linux/macOS
   echo "SESSION_SECRET=$(openssl rand -hex 32)" > .env.local
   ```

3. **Launch**:
   ```bash
   npm run dev
   ```
   Open [http://localhost:3000](http://localhost:3000)

---

## Contributing
Contributions are what make the open-source community an amazing place! If you have a suggestion that would make this better, please fork the repo and create a pull request.

---
*Developed with ❤️ by okcoe21*
