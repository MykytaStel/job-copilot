import { sqliteTable, text, integer } from 'drizzle-orm/sqlite-core';

export const profiles = sqliteTable('profiles', {
  id: text('id').primaryKey(),
  name: text('name').notNull(),
  email: text('email').notNull(),
  location: text('location'),
  summary: text('summary'),
  skills: text('skills').notNull(), // JSON array string
  updatedAt: text('updated_at').notNull(),
});

export const jobs = sqliteTable('jobs', {
  id: text('id').primaryKey(),
  source: text('source').notNull(),
  url: text('url'),
  title: text('title').notNull(),
  company: text('company').notNull(),
  description: text('description').notNull(),
  notes: text('notes').notNull().default(''),
  createdAt: text('created_at').notNull(),
});

export const resumes = sqliteTable('resumes', {
  id: text('id').primaryKey(),
  version: integer('version').notNull(),
  filename: text('filename').notNull(),
  rawText: text('raw_text').notNull(),
  isActive: integer('is_active', { mode: 'boolean' }).notNull().default(false),
  uploadedAt: text('uploaded_at').notNull(),
});

export const matchResults = sqliteTable('match_results', {
  id: text('id').primaryKey(),
  jobId: text('job_id').notNull(),
  resumeId: text('resume_id').notNull(),
  score: integer('score').notNull(),
  matchedSkills: text('matched_skills').notNull(), // JSON
  missingSkills: text('missing_skills').notNull(), // JSON
  notes: text('notes').notNull(),
  createdAt: text('created_at').notNull(),
});

export const applications = sqliteTable('applications', {
  id: text('id').primaryKey(),
  jobId: text('job_id').notNull().unique(),
  resumeId: text('resume_id'),
  status: text('status').notNull().default('saved'),
  appliedAt: text('applied_at'),
  dueDate: text('due_date'),
  updatedAt: text('updated_at').notNull(),
});

export const applicationNotes = sqliteTable('application_notes', {
  id: text('id').primaryKey(),
  applicationId: text('application_id').notNull(),
  content: text('content').notNull(),
  createdAt: text('created_at').notNull(),
});

export const contacts = sqliteTable('contacts', {
  id: text('id').primaryKey(),
  name: text('name').notNull(),
  email: text('email'),
  phone: text('phone'),
  linkedinUrl: text('linkedin_url'),
  company: text('company'),
  role: text('role'),
  createdAt: text('created_at').notNull(),
});

export const applicationContacts = sqliteTable('application_contacts', {
  id: text('id').primaryKey(),
  applicationId: text('application_id').notNull(),
  contactId: text('contact_id').notNull(),
  relationship: text('relationship').notNull().default('recruiter'),
});

export const activities = sqliteTable('activities', {
  id: text('id').primaryKey(),
  applicationId: text('application_id').notNull(),
  type: text('type').notNull(),
  description: text('description').notNull(),
  happenedAt: text('happened_at').notNull(),
  createdAt: text('created_at').notNull(),
});

export const tasks = sqliteTable('tasks', {
  id: text('id').primaryKey(),
  applicationId: text('application_id').notNull(),
  title: text('title').notNull(),
  remindAt: text('remind_at'),
  done: integer('done', { mode: 'boolean' }).notNull().default(false),
  createdAt: text('created_at').notNull(),
});

export const alerts = sqliteTable('alerts', {
  id: text('id').primaryKey(),
  keywords: text('keywords').notNull(), // JSON array
  telegramChatId: text('telegram_chat_id').notNull(),
  active: integer('active', { mode: 'boolean' }).notNull().default(true),
  createdAt: text('created_at').notNull(),
});

export const coverLetters = sqliteTable('cover_letters', {
  id: text('id').primaryKey(),
  jobId: text('job_id').notNull(),
  content: text('content').notNull(),
  tone: text('tone').notNull().default('formal'),
  createdAt: text('created_at').notNull(),
});

export const interviewQA = sqliteTable('interview_qa', {
  id: text('id').primaryKey(),
  jobId: text('job_id').notNull(),
  question: text('question').notNull(),
  answer: text('answer').notNull().default(''),
  category: text('category').notNull().default('behavioral'),
  createdAt: text('created_at').notNull(),
});

export const offers = sqliteTable('offers', {
  id: text('id').primaryKey(),
  jobId: text('job_id').notNull().unique(),
  salary: integer('salary'),
  currency: text('currency').notNull().default('UAH'),
  equity: text('equity'),
  benefits: text('benefits').notNull().default('[]'), // JSON array
  deadline: text('deadline'),
  notes: text('notes').notNull().default(''),
  createdAt: text('created_at').notNull(),
});
