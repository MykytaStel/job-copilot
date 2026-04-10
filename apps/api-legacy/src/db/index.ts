import Database from 'better-sqlite3';
import { drizzle } from 'drizzle-orm/better-sqlite3';
import { mkdirSync } from 'fs';
import * as schema from './schema.js';

mkdirSync('./data', { recursive: true });

const sqlite = new Database('./data/db.sqlite');

// Enable WAL for better concurrent read performance
sqlite.pragma('journal_mode = WAL');

// Migrations: add new columns to existing tables (safe to run multiple times)
try { sqlite.exec(`ALTER TABLE jobs ADD COLUMN notes TEXT NOT NULL DEFAULT ''`); } catch {}
try { sqlite.exec(`ALTER TABLE applications ADD COLUMN due_date TEXT`); } catch {}

export const db = drizzle(sqlite, { schema });

// Create tables if they don't exist (no migrations needed for dev)
sqlite.exec(`
  CREATE TABLE IF NOT EXISTS profiles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    location TEXT,
    summary TEXT,
    skills TEXT NOT NULL,
    updated_at TEXT NOT NULL
  );

  CREATE TABLE IF NOT EXISTS jobs (
    id TEXT PRIMARY KEY,
    source TEXT NOT NULL,
    url TEXT,
    title TEXT NOT NULL,
    company TEXT NOT NULL,
    description TEXT NOT NULL,
    created_at TEXT NOT NULL
  );

  CREATE TABLE IF NOT EXISTS resumes (
    id TEXT PRIMARY KEY,
    version INTEGER NOT NULL,
    filename TEXT NOT NULL,
    raw_text TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 0,
    uploaded_at TEXT NOT NULL
  );

  CREATE TABLE IF NOT EXISTS match_results (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL,
    resume_id TEXT NOT NULL,
    score INTEGER NOT NULL,
    matched_skills TEXT NOT NULL,
    missing_skills TEXT NOT NULL,
    notes TEXT NOT NULL,
    created_at TEXT NOT NULL
  );

  CREATE TABLE IF NOT EXISTS applications (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL UNIQUE,
    resume_id TEXT,
    status TEXT NOT NULL DEFAULT 'saved',
    applied_at TEXT,
    updated_at TEXT NOT NULL
  );

  CREATE TABLE IF NOT EXISTS application_notes (
    id TEXT PRIMARY KEY,
    application_id TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL
  );

  CREATE TABLE IF NOT EXISTS alerts (
    id TEXT PRIMARY KEY,
    keywords TEXT NOT NULL,
    telegram_chat_id TEXT NOT NULL,
    active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL
  );

  CREATE TABLE IF NOT EXISTS contacts (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT,
    phone TEXT,
    linkedin_url TEXT,
    company TEXT,
    role TEXT,
    created_at TEXT NOT NULL
  );

  CREATE TABLE IF NOT EXISTS application_contacts (
    id TEXT PRIMARY KEY,
    application_id TEXT NOT NULL,
    contact_id TEXT NOT NULL,
    relationship TEXT NOT NULL DEFAULT 'recruiter'
  );

  CREATE TABLE IF NOT EXISTS activities (
    id TEXT PRIMARY KEY,
    application_id TEXT NOT NULL,
    type TEXT NOT NULL,
    description TEXT NOT NULL,
    happened_at TEXT NOT NULL,
    created_at TEXT NOT NULL
  );

  CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    application_id TEXT NOT NULL,
    title TEXT NOT NULL,
    remind_at TEXT,
    done INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
  );

  CREATE TABLE IF NOT EXISTS cover_letters (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL,
    content TEXT NOT NULL,
    tone TEXT NOT NULL DEFAULT 'formal',
    created_at TEXT NOT NULL
  );

  CREATE TABLE IF NOT EXISTS interview_qa (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL,
    question TEXT NOT NULL,
    answer TEXT NOT NULL DEFAULT '',
    category TEXT NOT NULL DEFAULT 'behavioral',
    created_at TEXT NOT NULL
  );

  CREATE TABLE IF NOT EXISTS offers (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL UNIQUE,
    salary INTEGER,
    currency TEXT NOT NULL DEFAULT 'UAH',
    equity TEXT,
    benefits TEXT NOT NULL DEFAULT '[]',
    deadline TEXT,
    notes TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL
  );
`);
