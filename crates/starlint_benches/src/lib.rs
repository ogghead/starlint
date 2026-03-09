//! Benchmark utilities and fixtures for starlint rule benchmarks.
//!
//! Provides pre-parsed fixtures and helpers for criterion-based benchmarks.
//! Run individual benchmarks with: `cargo bench -p starlint_benches -- <filter>`
//!
//! # Examples
//!
//! ```bash
//! cargo bench -p starlint_benches                    # all rules (~30 min)
//! cargo bench -p starlint_benches -- core/           # core plugin only
//! cargo bench -p starlint_benches -- core/no-debugger # single rule
//! cargo bench -p starlint_benches -- _bundle         # all plugin bundles
//! cargo bench -p starlint_benches -- parse/          # parse-only benchmarks
//! ```

#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::print_stdout)]
#![allow(clippy::print_stderr)]

use std::path::PathBuf;

use starlint_ast::tree::AstTree;
use starlint_parser::{ParseOptions, ParseResult};
use starlint_rule_framework::FileContext;
use starlint_scope::ScopeData;

/// A pre-parsed source file ready for benchmarking.
///
/// Holds the source text, parsed AST, and scope analysis so benchmark loops
/// measure only rule execution time.
pub struct ParsedFixture {
    /// The source text.
    pub source: String,
    /// The parsed AST tree.
    pub tree: AstTree,
    /// Pre-computed scope data (always built so semantic rules can run).
    pub scope_data: ScopeData,
    /// The virtual file path (determines parse options via extension).
    pub file_path: PathBuf,
}

impl ParsedFixture {
    /// Parse `source` as if it came from `file_path`.
    ///
    /// Automatically infers JSX/TypeScript/module mode from the file extension
    /// and pre-builds scope data for semantic rules.
    #[must_use]
    pub fn new(source: &str, file_path: &str) -> Self {
        let path = PathBuf::from(file_path);
        let options = ParseOptions::from_path(&path);
        let ParseResult { tree, .. } = starlint_parser::parse(source, options);
        let scope_data = starlint_scope::build_scope_data(&tree);
        Self {
            source: source.to_owned(),
            tree,
            scope_data,
            file_path: path,
        }
    }

    /// Build a [`FileContext`] referencing this fixture's data.
    #[must_use]
    pub fn file_context(&self) -> FileContext<'_> {
        let extension = self
            .file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("js");
        FileContext {
            file_path: &self.file_path,
            source_text: &self.source,
            extension,
            tree: &self.tree,
            scope_data: Some(&self.scope_data),
        }
    }
}

// ─── Fixtures ────────────────────────────────────────────────────────────────
//
// Each fixture simulates a realistic application file (~500-800 lines).
// The goal is a representative noise-to-signal ratio: most code is clean,
// with anti-patterns scattered naturally. This exercises AST traversal
// realistically — rules must walk thousands of nodes to find the few matches.

/// General JavaScript source (~600 lines).
///
/// Simulates a Node.js data processing service with classes, async functions,
/// promise chains, error handling, utility functions, and configuration.
/// Heavy on `CallExpression`, `BinaryExpression`, `ArrowFunctionExpression`,
/// `VariableDeclarator`, `IfStatement`, `ForStatement`, and `SwitchStatement`.
pub const JS_FIXTURE: &str = r"
import { readFile, writeFile } from 'fs/promises';
import path from 'path';
import * as crypto from 'crypto';
import { EventEmitter } from 'events';
import { Transform } from 'stream';
import defaultExport from './default';
import { validateInput, sanitizeString, formatDate } from './helpers';
import { DatabasePool, createConnection } from './database';
import { Logger } from './logger';
import { MetricsCollector } from './metrics';

// ── Constants ────────────────────────────────────────────────────────────────

const MAX_RETRIES = 3;
const DEFAULT_TIMEOUT = 5000;
const BATCH_SIZE = 100;
const CACHE_TTL = 60 * 60 * 1000;
const RATE_LIMIT = 1000;
const VERSION = '2.4.1';
const MAGIC = 42;
const PI_APPROX = 3.14;
const STATUS_CODES = {
    OK: 200,
    CREATED: 201,
    BAD_REQUEST: 400,
    UNAUTHORIZED: 401,
    NOT_FOUND: 404,
    INTERNAL_ERROR: 500,
};

var globalState = null;
let mutableCounter = 0;
let requestCount = 0;

// ── Utility functions ────────────────────────────────────────────────────────

function generateId(prefix) {
    const timestamp = Date.now().toString(36);
    const random = Math.random().toString(36).substring(2, 10);
    return `${prefix}_${timestamp}_${random}`;
}

function clamp(value, min, max) {
    if (value < min) return min;
    if (value > max) return max;
    return value;
}

function debounce(fn, delay) {
    let timer = null;
    return function (...args) {
        if (timer !== null) {
            clearTimeout(timer);
        }
        timer = setTimeout(() => {
            fn.apply(this, args);
            timer = null;
        }, delay);
    };
}

function deepMerge(target, source) {
    const result = { ...target };
    for (const key in source) {
        if (Object.prototype.hasOwnProperty.call(source, key)) {
            if (
                typeof source[key] === 'object' &&
                source[key] !== null &&
                !Array.isArray(source[key])
            ) {
                result[key] = deepMerge(result[key] || {}, source[key]);
            } else {
                result[key] = source[key];
            }
        }
    }
    return result;
}

function retry(fn, maxAttempts, delayMs) {
    return async function (...args) {
        let lastError;
        for (let attempt = 1; attempt <= maxAttempts; attempt++) {
            try {
                const result = await fn.apply(this, args);
                return result;
            } catch (error) {
                lastError = error;
                if (attempt < maxAttempts) {
                    const backoff = delayMs * Math.pow(2, attempt - 1);
                    await new Promise(resolve => setTimeout(resolve, backoff));
                }
            }
        }
        throw lastError;
    };
}

function parseQueryString(qs) {
    if (!qs || typeof qs != 'string') {
        return {};
    }
    const params = {};
    const pairs = qs.replace(/^\?/, '').split('&');
    for (var i = 0; i < pairs.length; i++) {
        const [key, value] = pairs[i].split('=');
        if (key) {
            params[decodeURIComponent(key)] = value
                ? decodeURIComponent(value)
                : '';
        }
    }
    return params;
}

function flattenObject(obj, prefix, separator) {
    const result = {};
    for (const key in obj) {
        const fullKey = prefix ? `${prefix}${separator || '.'}${key}` : key;
        if (typeof obj[key] === 'object' && obj[key] !== null && !Array.isArray(obj[key])) {
            Object.assign(result, flattenObject(obj[key], fullKey, separator));
        } else {
            result[fullKey] = obj[key];
        }
    }
    return result;
}

const identity = x => x;
const noop = () => {};
const pipe = (...fns) => (x) => fns.reduce((acc, fn) => fn(acc), x);
const compose = (...fns) => (x) => fns.reduceRight((acc, fn) => fn(acc), x);

// ── Configuration ────────────────────────────────────────────────────────────

class Config {
    constructor(defaults) {
        this.values = { ...defaults };
        this.overrides = new Map();
        this.listeners = [];
    }

    get(key) {
        if (this.overrides.has(key)) {
            return this.overrides.get(key);
        }
        const parts = key.split('.');
        let current = this.values;
        for (const part of parts) {
            if (current == null || typeof current !== 'object') {
                return undefined;
            }
            current = current[part];
        }
        return current;
    }

    set(key, value) {
        this.overrides.set(key, value);
        for (const listener of this.listeners) {
            listener(key, value);
        }
    }

    onChange(listener) {
        this.listeners.push(listener);
        return () => {
            const index = this.listeners.indexOf(listener);
            if (index >= 0) {
                this.listeners.splice(index, 1);
            }
        };
    }

    toJSON() {
        const merged = { ...this.values };
        for (const [key, value] of this.overrides) {
            merged[key] = value;
        }
        return merged;
    }
}

// ── Cache ────────────────────────────────────────────────────────────────────

class LRUCache {
    constructor(maxSize, ttl) {
        this.maxSize = maxSize || 1000;
        this.ttl = ttl || CACHE_TTL;
        this.cache = new Map();
        this.hits = 0;
        this.misses = 0;
    }

    get(key) {
        const entry = this.cache.get(key);
        if (!entry) {
            this.misses += 1;
            return undefined;
        }
        if (Date.now() - entry.timestamp > this.ttl) {
            this.cache.delete(key);
            this.misses += 1;
            return undefined;
        }
        this.cache.delete(key);
        this.cache.set(key, { ...entry, timestamp: Date.now() });
        this.hits += 1;
        return entry.value;
    }

    set(key, value) {
        if (this.cache.has(key)) {
            this.cache.delete(key);
        }
        if (this.cache.size >= this.maxSize) {
            const oldest = this.cache.keys().next().value;
            this.cache.delete(oldest);
        }
        this.cache.set(key, { value, timestamp: Date.now() });
    }

    has(key) {
        return this.cache.has(key) &&
            Date.now() - this.cache.get(key).timestamp <= this.ttl;
    }

    clear() {
        this.cache.clear();
        this.hits = 0;
        this.misses = 0;
    }

    get size() {
        return this.cache.size;
    }

    get hitRate() {
        const total = this.hits + this.misses;
        return total === 0 ? 0 : this.hits / total;
    }
}

// ── Data processing pipeline ─────────────────────────────────────────────────

class DataProcessor extends EventEmitter {
    constructor(config) {
        super();
        this.config = config;
        this.cache = new LRUCache(config.get('cache.maxSize'), config.get('cache.ttl'));
        this.pipeline = [];
        this.isRunning = false;
        this.processedCount = 0;
        this.errorCount = 0;
        this.initialized = false;
    }

    async initialize() {
        if (this.initialized) return;
        this.initialized = true;
        const defaults = await import('./defaults');
        Object.assign(this.config.values, defaults);
        this.emit('initialized');
    }

    addStage(name, fn, options) {
        this.pipeline.push({
            name,
            fn,
            timeout: (options && options.timeout) || DEFAULT_TIMEOUT,
            retries: (options && options.retries) || 0,
            enabled: options ? options.enabled !== false : true,
        });
        return this;
    }

    async processItem(item) {
        const id = generateId('item');
        let current = { ...item, _id: id, _startedAt: Date.now() };

        for (const stage of this.pipeline) {
            if (!stage.enabled) continue;

            try {
                const fn = stage.retries > 0
                    ? retry(stage.fn, stage.retries, 100)
                    : stage.fn;

                const result = await Promise.race([
                    fn(current),
                    new Promise((_, reject) =>
                        setTimeout(() => reject(new Error(`Stage ${stage.name} timed out`)), stage.timeout)
                    ),
                ]);

                if (result === null || result === undefined) {
                    this.emit('skip', { id, stage: stage.name });
                    return null;
                }

                current = typeof result === 'object' ? { ...current, ...result } : current;
            } catch (error) {
                this.errorCount += 1;
                this.emit('error', { id, stage: stage.name, error });

                if (this.config.get('pipeline.stopOnError') !== false) {
                    throw error;
                }
            }
        }

        this.processedCount += 1;
        current._completedAt = Date.now();
        current._duration = current._completedAt - current._startedAt;
        this.emit('processed', current);
        return current;
    }

    async processBatch(items) {
        const batchSize = this.config.get('pipeline.batchSize') || BATCH_SIZE;
        const results = [];

        for (let i = 0; i < items.length; i += batchSize) {
            const batch = items.slice(i, i + batchSize);
            const batchResults = await Promise.all(
                batch.map(item => this.processItem(item).catch(err => {
                    console.error(`Failed to process item: ${err.message}`);
                    return null;
                }))
            );
            results.push(...batchResults.filter(Boolean));

            if (i + batchSize < items.length) {
                await new Promise(resolve => setTimeout(resolve, 10));
            }
        }

        return results;
    }

    getStats() {
        return {
            processed: this.processedCount,
            errors: this.errorCount,
            cacheSize: this.cache.size,
            cacheHitRate: this.cache.hitRate,
            stages: this.pipeline.length,
            activeStages: this.pipeline.filter(s => s.enabled).length,
        };
    }

    reset() {
        this.processedCount = 0;
        this.errorCount = 0;
        this.cache.clear();
        this.isRunning = false;
    }
}

// ── HTTP client ──────────────────────────────────────────────────────────────

async function fetchWithRetry(url, options) {
    const maxAttempts = (options && options.retries) || MAX_RETRIES;
    let lastError;

    for (let attempt = 0; attempt < maxAttempts; attempt++) {
        try {
            const response = await fetch(url, {
                method: (options && options.method) || 'GET',
                headers: {
                    'Content-Type': 'application/json',
                    ...((options && options.headers) || {}),
                },
                body: options && options.body ? JSON.stringify(options.body) : undefined,
                signal: options && options.signal,
            });

            if (!response.ok) {
                const statusCode = response.status;
                if (statusCode >= 400 && statusCode < 500) {
                    const body = await response.text();
                    throw new Error(`Client error ${statusCode}: ${body}`);
                }
                throw new Error(`Server error ${statusCode}`);
            }

            const contentType = response.headers.get('content-type');
            if (contentType && contentType.includes('application/json')) {
                return await response.json();
            }
            return await response.text();
        } catch (error) {
            lastError = error;
            if (attempt < maxAttempts - 1) {
                const delay = DEFAULT_TIMEOUT * Math.pow(2, attempt);
                await new Promise(resolve => setTimeout(resolve, delay));
            }
        }
    }

    throw lastError;
}

// ── Validation ───────────────────────────────────────────────────────────────

function validateRecord(record, schema) {
    const errors = [];

    for (const field of schema.required || []) {
        if (record[field] === undefined || record[field] === null) {
            errors.push({ field, message: `${field} is required` });
        }
    }

    for (const [field, rules] of Object.entries(schema.fields || {})) {
        const value = record[field];
        if (value === undefined) continue;

        if (rules.type && typeof value !== rules.type) {
            errors.push({ field, message: `${field} must be type ${rules.type}` });
        }

        if (rules.minLength && typeof value === 'string' && value.length < rules.minLength) {
            errors.push({ field, message: `${field} must be at least ${rules.minLength} chars` });
        }

        if (rules.maxLength && typeof value === 'string' && value.length > rules.maxLength) {
            errors.push({ field, message: `${field} must be at most ${rules.maxLength} chars` });
        }

        if (rules.min !== undefined && typeof value === 'number' && value < rules.min) {
            errors.push({ field, message: `${field} must be >= ${rules.min}` });
        }

        if (rules.max !== undefined && typeof value === 'number' && value > rules.max) {
            errors.push({ field, message: `${field} must be <= ${rules.max}` });
        }

        if (rules.pattern && typeof value === 'string' && !rules.pattern.test(value)) {
            errors.push({ field, message: `${field} does not match pattern` });
        }

        if (rules.enum && !rules.enum.includes(value)) {
            errors.push({ field, message: `${field} must be one of: ${rules.enum.join(', ')}` });
        }
    }

    return { valid: errors.length === 0, errors };
}

// ── Transforms ───────────────────────────────────────────────────────────────

const transforms = {
    uppercase: (s) => typeof s === 'string' ? s.toUpperCase() : s,
    lowercase: (s) => typeof s === 'string' ? s.toLowerCase() : s,
    trim: (s) => typeof s === 'string' ? s.trim() : s,
    parseInt: (s) => {
        const n = Number.parseInt(s, 10);
        return Number.isNaN(n) ? null : n;
    },
    parseFloat: (s) => {
        const n = Number.parseFloat(s);
        return Number.isNaN(n) ? null : n;
    },
    toArray: (s) => Array.isArray(s) ? s : [s],
    compact: (arr) => Array.isArray(arr) ? arr.filter(Boolean) : arr,
    unique: (arr) => Array.isArray(arr) ? [...new Set(arr)] : arr,
    sort: (arr) => Array.isArray(arr) ? [...arr].sort() : arr,
    reverse: (arr) => Array.isArray(arr) ? [...arr].reverse() : arr,
    flatten: (arr) => Array.isArray(arr) ? arr.flat(Infinity) : arr,
};

function applyTransforms(data, transformList) {
    let result = data;
    for (const name of transformList) {
        const fn = transforms[name];
        if (fn) {
            result = fn(result);
        }
    }
    return result;
}

// ── Generators ───────────────────────────────────────────────────────────────

function* range(start, end, step) {
    const s = step || 1;
    for (let i = start; i < end; i += s) {
        yield i;
    }
}

function* chunk(arr, size) {
    for (let i = 0; i < arr.length; i += size) {
        yield arr.slice(i, i + size);
    }
}

async function* streamLines(filePath) {
    const content = await readFile(filePath, 'utf-8');
    const lines = content.split('\n');
    for (const line of lines) {
        yield line;
    }
}

// ── Main entry point ─────────────────────────────────────────────────────────

async function main() {
    debugger;

    const config = new Config({
        cache: { maxSize: 5000, ttl: CACHE_TTL },
        pipeline: { batchSize: BATCH_SIZE, stopOnError: false },
        http: { timeout: DEFAULT_TIMEOUT, retries: MAX_RETRIES },
    });

    const processor = new DataProcessor(config);
    await processor.initialize();

    processor.addStage('validate', (item) => {
        const result = validateRecord(item, {
            required: ['name', 'email'],
            fields: {
                name: { type: 'string', minLength: 1, maxLength: 200 },
                email: { type: 'string', pattern: /^[^@]+@[^@]+$/ },
                age: { type: 'number', min: 0, max: 150 },
            },
        });
        if (!result.valid) {
            throw new Error(`Validation failed: ${result.errors.map(e => e.message).join(', ')}`);
        }
        return item;
    });

    processor.addStage('transform', (item) => ({
        ...item,
        name: applyTransforms(item.name, ['trim', 'lowercase']),
        email: applyTransforms(item.email, ['trim', 'lowercase']),
        tags: applyTransforms(item.tags || [], ['compact', 'unique', 'sort']),
    }));

    processor.addStage('enrich', async (item) => {
        const cached = processor.cache.get(item.email);
        if (cached) return { ...item, ...cached };

        const userData = await fetchWithRetry(`/api/users?email=${encodeURIComponent(item.email)}`);
        processor.cache.set(item.email, userData);
        return { ...item, ...userData };
    }, { retries: 2, timeout: 10000 });

    processor.on('processed', (item) => {
        requestCount += 1;
        mutableCounter = mutableCounter + 1;
    });

    processor.on('error', ({ id, stage, error }) => {
        console.error(`Error in ${stage} for ${id}: ${error.message}`);
    });

    try {
        const inputData = await fetchWithRetry('/api/input-data');
        const items = Array.isArray(inputData) ? inputData : [inputData];
        const results = await processor.processBatch(items);
        const stats = processor.getStats();

        console.log(`Processed ${stats.processed} items with ${stats.errors} errors`);
        console.log(`Cache hit rate: ${(stats.cacheHitRate * 100).toFixed(1)}%`);

        await writeFile(
            path.join('/tmp', 'output.json'),
            JSON.stringify(results, null, 2)
        );
    } catch (err) {
        console.error('Pipeline failed:', err);
        process.exit(1);
    }

    switch (globalState) {
        case 'ready':
            mutableCounter += 1;
            break;
        case 'error':
            console.warn('Pipeline in error state');
            break;
        case 'ready':
            console.log('duplicate case');
            break;
        default:
            console.warn('unknown state');
    }

    try {
        eval('var x = 1');
    } catch (err) {
        // empty catch
    }

    void 0;
}

const arr = [1, 2, , 4, 5];
const nested = true ? (false ? 'a' : 'b') : 'c';

class EmptyClass {}

export { DataProcessor, Config, LRUCache, main, transforms, validateRecord };
export default DataProcessor;
";

/// React/TSX source (~550 lines).
///
/// Simulates a real dashboard with multiple components, custom hooks,
/// forms, data tables, modals, and accessibility patterns.
/// Heavy on `JSXOpeningElement`, `JSXAttribute`, `JSXElement`,
/// `CallExpression` (hooks), `ArrowFunctionExpression`.
pub const TSX_FIXTURE: &str = r#"
import React, {
    useState,
    useEffect,
    useMemo,
    useCallback,
    useRef,
    useContext,
    useReducer,
    createContext,
    forwardRef,
    memo,
} from 'react';

// ── Types ────────────────────────────────────────────────────────────────────

interface User {
    id: number;
    name: string;
    email: string;
    role: 'admin' | 'editor' | 'viewer';
    avatar?: string;
    lastLogin: string;
    isActive: boolean;
}

interface PaginationState {
    page: number;
    pageSize: number;
    total: number;
}

interface FilterState {
    search: string;
    role: string;
    status: string;
    sortBy: string;
    sortOrder: 'asc' | 'desc';
}

interface ModalProps {
    isOpen: boolean;
    onClose: () => void;
    title: string;
    children: React.ReactNode;
    size?: 'sm' | 'md' | 'lg';
}

interface ButtonProps {
    label: string;
    onClick: () => void;
    disabled?: boolean;
    variant?: 'primary' | 'secondary' | 'danger';
    size?: 'sm' | 'md' | 'lg';
    icon?: React.ReactNode;
    loading?: boolean;
    children?: React.ReactNode;
}

interface TableColumn<T> {
    key: keyof T;
    label: string;
    sortable?: boolean;
    render?: (value: unknown, row: T) => React.ReactNode;
}

type Action =
    | { type: 'SET_FILTERS'; payload: Partial<FilterState> }
    | { type: 'SET_PAGE'; payload: number }
    | { type: 'RESET' };

// ── Context ──────────────────────────────────────────────────────────────────

interface ThemeContextValue {
    mode: 'light' | 'dark';
    toggle: () => void;
    primaryColor: string;
}

const ThemeContext = createContext<ThemeContextValue>({
    mode: 'light',
    toggle: () => {},
    primaryColor: '#3b82f6',
});

// ── Custom hooks ─────────────────────────────────────────────────────────────

function useDebounce<T>(value: T, delay: number): T {
    const [debounced, setDebounced] = useState(value);

    useEffect(() => {
        const timer = setTimeout(() => setDebounced(value), delay);
        return () => clearTimeout(timer);
    }, [value, delay]);

    return debounced;
}

function useLocalStorage<T>(key: string, initialValue: T): [T, (v: T) => void] {
    const [stored, setStored] = useState<T>(() => {
        try {
            const item = window.localStorage.getItem(key);
            return item ? JSON.parse(item) : initialValue;
        } catch {
            return initialValue;
        }
    });

    const setValue = useCallback(
        (value: T) => {
            setStored(value);
            window.localStorage.setItem(key, JSON.stringify(value));
        },
        [key],
    );

    return [stored, setValue];
}

function useFetch<T>(url: string) {
    const [data, setData] = useState<T | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const abortRef = useRef<AbortController | null>(null);

    useEffect(() => {
        abortRef.current = new AbortController();
        const signal = abortRef.current.signal;

        setLoading(true);
        fetch(url, { signal })
            .then(res => {
                if (!res.ok) throw new Error(`HTTP ${res.status}`);
                return res.json();
            })
            .then(json => {
                setData(json);
                setLoading(false);
            })
            .catch(err => {
                if (err.name !== 'AbortError') {
                    setError(err.message);
                    setLoading(false);
                }
            });

        return () => {
            abortRef.current?.abort();
        };
    }, [url]);

    return { data, loading, error };
}

// ── Reducer ──────────────────────────────────────────────────────────────────

const initialFilters: FilterState = {
    search: '',
    role: '',
    status: '',
    sortBy: 'name',
    sortOrder: 'asc',
};

function filterReducer(state: FilterState, action: Action): FilterState {
    switch (action.type) {
        case 'SET_FILTERS':
            return { ...state, ...action.payload };
        case 'SET_PAGE':
            return state;
        case 'RESET':
            return initialFilters;
        default:
            return state;
    }
}

// ── Components ───────────────────────────────────────────────────────────────

const Button: React.FC<ButtonProps> = ({
    label,
    onClick,
    disabled = false,
    variant = 'primary',
    size = 'md',
    icon,
    loading = false,
    children,
}) => {
    const classNames = [
        'btn',
        `btn-${variant}`,
        `btn-${size}`,
        disabled ? 'btn-disabled' : '',
        loading ? 'btn-loading' : '',
    ]
        .filter(Boolean)
        .join(' ');

    return (
        <button
            type="button"
            className={classNames}
            onClick={onClick}
            disabled={disabled || loading}
            aria-busy={loading}
        >
            {icon && <span className="btn-icon">{icon}</span>}
            {loading ? 'Loading...' : label}
            {children}
        </button>
    );
};

function Modal({ isOpen, onClose, title, children, size = 'md' }: ModalProps) {
    const overlayRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (!isOpen) return;

        const handleEsc = (e: KeyboardEvent) => {
            if (e.key === 'Escape') onClose();
        };
        document.addEventListener('keydown', handleEsc);
        return () => document.removeEventListener('keydown', handleEsc);
    }, [isOpen, onClose]);

    if (!isOpen) return null;

    return (
        <div
            ref={overlayRef}
            className="modal-overlay"
            role="dialog"
            aria-modal="true"
            aria-labelledby="modal-title"
            onClick={(e) => {
                if (e.target === overlayRef.current) onClose();
            }}
        >
            <div className={`modal-content modal-${size}`}>
                <div className="modal-header">
                    <h2 id="modal-title">{title}</h2>
                    <button
                        type="button"
                        className="modal-close"
                        onClick={onClose}
                        aria-label="Close modal"
                    >
                        x
                    </button>
                </div>
                <div className="modal-body">{children}</div>
            </div>
        </div>
    );
}

const Badge = memo(function Badge({ text, color }: { text: string; color: string }) {
    return (
        <span className="badge" style={{ backgroundColor: color }}>
            {text}
        </span>
    );
});

const TextInput = forwardRef<HTMLInputElement, {
    label: string;
    value: string;
    onChange: (v: string) => void;
    placeholder?: string;
    error?: string;
}>(function TextInput({ label, value, onChange, placeholder, error }, ref) {
    const id = `input-${label.toLowerCase().replace(/\s+/g, '-')}`;
    return (
        <div className="form-group">
            <label htmlFor={id}>{label}</label>
            <input
                ref={ref}
                id={id}
                type="text"
                value={value}
                onChange={(e) => onChange(e.target.value)}
                placeholder={placeholder}
                aria-invalid={!!error}
                aria-describedby={error ? `${id}-error` : undefined}
                className={error ? 'input-error' : ''}
            />
            {error && (
                <p id={`${id}-error`} role="alert" className="error-text">
                    {error}
                </p>
            )}
        </div>
    );
});

function SearchBar({ value, onChange }: { value: string; onChange: (v: string) => void }) {
    const inputRef = useRef<HTMLInputElement>(null);

    return (
        <div className="search-bar" role="search">
            <label htmlFor="search-input" className="sr-only">
                Search users
            </label>
            <input
                ref={inputRef}
                id="search-input"
                type="search"
                value={value}
                onChange={(e) => onChange(e.target.value)}
                placeholder="Search users..."
                aria-label="Search users"
            />
            {value && (
                <button
                    type="button"
                    onClick={() => {
                        onChange('');
                        inputRef.current?.focus();
                    }}
                    aria-label="Clear search"
                >
                    Clear
                </button>
            )}
        </div>
    );
}

function Pagination({
    page,
    pageSize,
    total,
    onPageChange,
}: PaginationState & { onPageChange: (p: number) => void }) {
    const totalPages = Math.ceil(total / pageSize);

    return (
        <nav aria-label="Pagination" className="pagination">
            <button
                type="button"
                onClick={() => onPageChange(page - 1)}
                disabled={page <= 1}
                aria-label="Previous page"
            >
                Previous
            </button>
            <span aria-current="page">
                Page {page} of {totalPages}
            </span>
            <button
                type="button"
                onClick={() => onPageChange(page + 1)}
                disabled={page >= totalPages}
                aria-label="Next page"
            >
                Next
            </button>
        </nav>
    );
}

function UserRow({ user, onEdit, onDelete }: { user: User; onEdit: (u: User) => void; onDelete: (id: number) => void }) {
    return (
        <tr>
            <td>
                <div className="user-cell">
                    <img
                        src={user.avatar || '/default-avatar.png'}
                        alt={`${user.name} avatar`}
                        width={32}
                        height={32}
                        className="avatar"
                    />
                    <span>{user.name}</span>
                </div>
            </td>
            <td>{user.email}</td>
            <td>
                <Badge
                    text={user.role}
                    color={user.role === 'admin' ? '#ef4444' : user.role === 'editor' ? '#f59e0b' : '#10b981'}
                />
            </td>
            <td>
                <Badge
                    text={user.isActive ? 'Active' : 'Inactive'}
                    color={user.isActive ? '#10b981' : '#6b7280'}
                />
            </td>
            <td>{new Date(user.lastLogin).toLocaleDateString()}</td>
            <td>
                <div className="actions">
                    <Button label="Edit" variant="secondary" size="sm" onClick={() => onEdit(user)} />
                    <Button label="Delete" variant="danger" size="sm" onClick={() => onDelete(user.id)} />
                </div>
            </td>
        </tr>
    );
}

// ── Main dashboard ───────────────────────────────────────────────────────────

function UserDashboard() {
    const theme = useContext(ThemeContext);
    const [filters, dispatch] = useReducer(filterReducer, initialFilters);
    const [pagination, setPagination] = useState<PaginationState>({ page: 1, pageSize: 20, total: 0 });
    const [selectedUser, setSelectedUser] = useState<User | null>(null);
    const [showModal, setShowModal] = useState(false);
    const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
    const [deleteId, setDeleteId] = useState<number | null>(null);
    const [preferences, setPreferences] = useLocalStorage('dashboard-prefs', { compact: false });

    const debouncedSearch = useDebounce(filters.search, 300);

    const queryUrl = useMemo(() => {
        const params = new URLSearchParams();
        params.set('page', String(pagination.page));
        params.set('size', String(pagination.pageSize));
        if (debouncedSearch) params.set('q', debouncedSearch);
        if (filters.role) params.set('role', filters.role);
        if (filters.status) params.set('status', filters.status);
        params.set('sort', filters.sortBy);
        params.set('order', filters.sortOrder);
        return `/api/users?${params.toString()}`;
    }, [debouncedSearch, filters, pagination.page, pagination.pageSize]);

    const { data, loading, error } = useFetch<{ users: User[]; total: number }>(queryUrl);

    useEffect(() => {
        if (data) {
            setPagination(prev => ({ ...prev, total: data.total }));
        }
    }, [data]);

    const handleEdit = useCallback((user: User) => {
        setSelectedUser(user);
        setShowModal(true);
    }, []);

    const handleDelete = useCallback((id: number) => {
        setDeleteId(id);
        setShowDeleteConfirm(true);
    }, []);

    const confirmDelete = useCallback(async () => {
        if (deleteId === null) return;
        try {
            await fetch(`/api/users/${deleteId}`, { method: 'DELETE' });
            setShowDeleteConfirm(false);
            setDeleteId(null);
        } catch (err) {
            console.error('Delete failed:', err);
        }
    }, [deleteId]);

    const handleSort = useCallback(
        (column: string) => {
            dispatch({
                type: 'SET_FILTERS',
                payload: {
                    sortBy: column,
                    sortOrder:
                        filters.sortBy === column && filters.sortOrder === 'asc'
                            ? 'desc'
                            : 'asc',
                },
            });
        },
        [filters.sortBy, filters.sortOrder],
    );

    if (error) {
        return (
            <div role="alert" className="error-container">
                <h2>Error loading users</h2>
                <p>{error}</p>
                <Button label="Retry" onClick={() => window.location.reload()} />
            </div>
        );
    }

    const users = data?.users || [];

    return (
        <div className={`dashboard ${theme.mode}`}>
            <header className="dashboard-header">
                <h1>User Management</h1>
                <div className="header-actions">
                    <Button
                        label={theme.mode === 'light' ? 'Dark Mode' : 'Light Mode'}
                        variant="secondary"
                        onClick={theme.toggle}
                    />
                    <Button label="Add User" variant="primary" onClick={() => setShowModal(true)} />
                </div>
            </header>

            <div className="toolbar">
                <SearchBar
                    value={filters.search}
                    onChange={(v) => dispatch({ type: 'SET_FILTERS', payload: { search: v } })}
                />
                <select
                    value={filters.role}
                    onChange={(e) => dispatch({ type: 'SET_FILTERS', payload: { role: e.target.value } })}
                    aria-label="Filter by role"
                >
                    <option value="">All Roles</option>
                    <option value="admin">Admin</option>
                    <option value="editor">Editor</option>
                    <option value="viewer">Viewer</option>
                </select>
                <select
                    value={filters.status}
                    onChange={(e) => dispatch({ type: 'SET_FILTERS', payload: { status: e.target.value } })}
                    aria-label="Filter by status"
                >
                    <option value="">All Statuses</option>
                    <option value="active">Active</option>
                    <option value="inactive">Inactive</option>
                </select>
                <Button
                    label="Reset Filters"
                    variant="secondary"
                    onClick={() => dispatch({ type: 'RESET' })}
                />
                <label className="compact-toggle">
                    <input
                        type="checkbox"
                        checked={preferences.compact}
                        onChange={(e) => setPreferences({ ...preferences, compact: e.target.checked })}
                    />
                    Compact View
                </label>
            </div>

            {loading ? (
                <div className="loading-spinner" aria-live="polite">
                    <p>Loading users...</p>
                </div>
            ) : (
                <>
                    <div className="results-info" aria-live="polite">
                        Showing {users.length} of {pagination.total} users
                    </div>

                    <table className={preferences.compact ? 'compact' : ''}>
                        <thead>
                            <tr>
                                {['name', 'email', 'role', 'status', 'lastLogin'].map((col) => (
                                    <th
                                        key={col}
                                        onClick={() => handleSort(col)}
                                        className={filters.sortBy === col ? 'sorted' : ''}
                                        aria-sort={
                                            filters.sortBy === col
                                                ? filters.sortOrder === 'asc'
                                                    ? 'ascending'
                                                    : 'descending'
                                                : 'none'
                                        }
                                    >
                                        {col.charAt(0).toUpperCase() + col.slice(1)}
                                        {filters.sortBy === col && (
                                            <span aria-hidden="true">
                                                {filters.sortOrder === 'asc' ? ' ^' : ' v'}
                                            </span>
                                        )}
                                    </th>
                                ))}
                                <th>Actions</th>
                            </tr>
                        </thead>
                        <tbody>
                            {users.map((user) => (
                                <UserRow
                                    key={user.id}
                                    user={user}
                                    onEdit={handleEdit}
                                    onDelete={handleDelete}
                                />
                            ))}
                        </tbody>
                    </table>

                    <Pagination
                        page={pagination.page}
                        pageSize={pagination.pageSize}
                        total={pagination.total}
                        onPageChange={(p) => setPagination(prev => ({ ...prev, page: p }))}
                    />
                </>
            )}

            <Modal isOpen={showModal} onClose={() => setShowModal(false)} title={selectedUser ? 'Edit User' : 'New User'} size="lg">
                <form onSubmit={(e) => e.preventDefault()}>
                    <TextInput label="Name" value={selectedUser?.name || ''} onChange={() => {}} placeholder="Enter name" />
                    <TextInput label="Email" value={selectedUser?.email || ''} onChange={() => {}} placeholder="Enter email" />
                    <div className="form-actions">
                        <Button label="Cancel" variant="secondary" onClick={() => setShowModal(false)} />
                        <Button label="Save" variant="primary" onClick={() => setShowModal(false)} />
                    </div>
                </form>
            </Modal>

            <Modal isOpen={showDeleteConfirm} onClose={() => setShowDeleteConfirm(false)} title="Confirm Delete" size="sm">
                <p>Are you sure you want to delete this user? This action cannot be undone.</p>
                <div className="form-actions">
                    <Button label="Cancel" variant="secondary" onClick={() => setShowDeleteConfirm(false)} />
                    <Button label="Delete" variant="danger" onClick={confirmDelete} />
                </div>
            </Modal>
        </div>
    );
}

// ── Error boundary ───────────────────────────────────────────────────────────

class ErrorBoundary extends React.Component<
    { children: React.ReactNode; fallback?: React.ReactNode },
    { hasError: boolean; error: Error | null }
> {
    state = { hasError: false, error: null as Error | null };

    static getDerivedStateFromError(error: Error) {
        return { hasError: true, error };
    }

    componentDidMount() {
        this.setState({ hasError: false });
    }

    render() {
        if (this.state.hasError) {
            return (
                this.props.fallback || (
                    <div role="alert">
                        <h1>Something went wrong</h1>
                        <p>{this.state.error?.message}</p>
                    </div>
                )
            );
        }
        return this.props.children;
    }
}

// ── App root ─────────────────────────────────────────────────────────────────

function App() {
    const [mode, setMode] = useState<'light' | 'dark'>('light');

    const themeValue = useMemo(
        () => ({
            mode,
            toggle: () => setMode(m => (m === 'light' ? 'dark' : 'light')),
            primaryColor: '#3b82f6',
        }),
        [mode],
    );

    return (
        <ThemeContext.Provider value={themeValue}>
            <ErrorBoundary>
                <UserDashboard />
            </ErrorBoundary>
        </ThemeContext.Provider>
    );
}

export default App;
export { UserDashboard, Button, Modal, ErrorBoundary, ThemeContext };
"#;

/// TypeScript source (~500 lines).
///
/// Simulates a real TypeScript service layer with complex types, generics,
/// enums, classes, utility types, type guards, overloads, and async patterns.
/// Heavy on `TSTypeReference`, `TSAsExpression`, `TSNonNullExpression`,
/// `TSEnumDeclaration`, `TSInterfaceDeclaration`, `TSTypeAliasDeclaration`.
pub const TS_FIXTURE: &str = r"
// ── Core domain types ────────────────────────────────────────────────────────

interface Entity {
    id: string;
    createdAt: Date;
    updatedAt: Date;
}

interface User extends Entity {
    name: string;
    email: string;
    age: number;
    role: UserRole;
    preferences: UserPreferences;
    readonly accountId: string;
}

interface UserPreferences {
    theme: 'light' | 'dark' | 'system';
    language: string;
    notifications: NotificationSettings;
    timezone?: string;
}

interface NotificationSettings {
    email: boolean;
    push: boolean;
    sms: boolean;
    frequency: 'immediate' | 'daily' | 'weekly';
}

interface Admin extends User {
    permissions: Permission[];
    department: string;
    level: number;
}

interface ApiResponse<T> {
    data: T;
    meta: {
        page: number;
        pageSize: number;
        total: number;
        hasMore: boolean;
    };
    errors: ApiError[];
}

interface ApiError {
    code: string;
    message: string;
    field?: string;
    details?: Record<string, unknown>;
}

interface PaginationParams {
    page?: number;
    pageSize?: number;
    sortBy?: string;
    sortOrder?: 'asc' | 'desc';
    filters?: Record<string, string | number | boolean>;
}

// ── Type utilities ───────────────────────────────────────────────────────────

type Nullable<T> = T | null;
type Optional<T> = T | undefined;
type DeepPartial<T> = { [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P] };
type DeepReadonly<T> = { readonly [P in keyof T]: T[P] extends object ? DeepReadonly<T[P]> : T[P] };
type RequireFields<T, K extends keyof T> = T & Required<Pick<T, K>>;
type ExcludeFields<T, K extends keyof T> = Omit<T, K>;
type FunctionType = Function;
type EmptyObj = {};
type StringOrNumber = string | number;
type EventHandler<T = void> = (event: T) => void;
type AsyncFunction<T, R> = (input: T) => Promise<R>;

type ExtractArrayType<T> = T extends Array<infer U> ? U : never;
type UnionToIntersection<U> = (U extends unknown ? (x: U) => void : never) extends (x: infer I) => void ? I : never;

interface EmptyInterface {}

// ── Enums ────────────────────────────────────────────────────────────────────

enum UserRole {
    Admin = 'ADMIN',
    Editor = 'EDITOR',
    Viewer = 'VIEWER',
    Guest = 'GUEST',
}

enum Permission {
    Read = 'read',
    Write = 'write',
    Delete = 'delete',
    Manage = 'manage',
    Export = 'export',
}

enum HttpStatus {
    Ok = 200,
    Created = 201,
    BadRequest = 400,
    Unauthorized = 401,
    Forbidden = 403,
    NotFound = 404,
    InternalError = 500,
}

enum MixedEnum {
    A = 0,
    B = 'b',
}

const enum CacheStrategy {
    None = 0,
    Memory = 1,
    Disk = 2,
    Both = 3,
}

enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
    Fatal = 4,
}

// ── Namespace ────────────────────────────────────────────────────────────────

namespace Validators {
    export function isEmail(value: string): boolean {
        return /^[^@]+@[^@]+\.[^@]+$/.test(value);
    }

    export function isNonEmpty(value: string): boolean {
        return value.trim().length > 0;
    }

    export function isInRange(value: number, min: number, max: number): boolean {
        return value >= min && value <= max;
    }
}

// ── Type guards ──────────────────────────────────────────────────────────────

function isUser(value: unknown): value is User {
    return (
        typeof value === 'object' &&
        value !== null &&
        'id' in value &&
        'name' in value &&
        'email' in value &&
        'role' in value
    );
}

function isAdmin(user: User): user is Admin {
    return 'permissions' in user && Array.isArray((user as Admin).permissions);
}

function isNonNullable<T>(value: T): value is NonNullable<T> {
    return value !== null && value !== undefined;
}

function assertDefined<T>(value: T | null | undefined, message?: string): asserts value is T {
    if (value === null || value === undefined) {
        throw new Error(message || 'Value is null or undefined');
    }
}

// ── Generic classes ──────────────────────────────────────────────────────────

class Repository<T extends Entity> {
    private items: Map<string, T> = new Map();
    private subscribers: Array<(event: string, item: T) => void> = [];

    async findById(id: string): Promise<T | undefined> {
        return this.items.get(id);
    }

    async findAll(params?: PaginationParams): Promise<ApiResponse<T[]>> {
        let results = Array.from(this.items.values());

        if (params?.sortBy) {
            const key = params.sortBy as keyof T;
            const order = params.sortOrder === 'desc' ? -1 : 1;
            results.sort((a, b) => {
                const aVal = a[key];
                const bVal = b[key];
                if (aVal < bVal) return -1 * order;
                if (aVal > bVal) return 1 * order;
                return 0;
            });
        }

        const page = params?.page || 1;
        const pageSize = params?.pageSize || 20;
        const start = (page - 1) * pageSize;
        const paged = results.slice(start, start + pageSize);

        return {
            data: paged,
            meta: {
                page,
                pageSize,
                total: results.length,
                hasMore: start + pageSize < results.length,
            },
            errors: [],
        };
    }

    async create(item: T): Promise<T> {
        this.items.set(item.id, item);
        this.notify('created', item);
        return item;
    }

    async update(id: string, updates: DeepPartial<T>): Promise<T | undefined> {
        const existing = this.items.get(id);
        if (!existing) return undefined;

        const updated = { ...existing, ...updates, updatedAt: new Date() } as T;
        this.items.set(id, updated);
        this.notify('updated', updated);
        return updated;
    }

    async delete(id: string): Promise<boolean> {
        const item = this.items.get(id);
        if (!item) return false;

        this.items.delete(id);
        this.notify('deleted', item);
        return true;
    }

    subscribe(handler: (event: string, item: T) => void): () => void {
        this.subscribers.push(handler);
        return () => {
            const index = this.subscribers.indexOf(handler);
            if (index >= 0) this.subscribers.splice(index, 1);
        };
    }

    private notify(event: string, item: T): void {
        for (const sub of this.subscribers) {
            sub(event, item);
        }
    }

    get count(): number {
        return this.items.size;
    }
}

class CacheableRepository<T extends Entity> extends Repository<T> {
    private cache: Map<string, { value: T; expires: number }> = new Map();
    private readonly ttl: number;

    constructor(ttlMs: number = 60000) {
        super();
        this.ttl = ttlMs;
    }

    override async findById(id: string): Promise<T | undefined> {
        const cached = this.cache.get(id);
        if (cached && cached.expires > Date.now()) {
            return cached.value;
        }
        this.cache.delete(id);

        const result = await super.findById(id);
        if (result) {
            this.cache.set(id, { value: result, expires: Date.now() + this.ttl });
        }
        return result;
    }

    clearCache(): void {
        this.cache.clear();
    }
}

// ── Service layer ────────────────────────────────────────────────────────────

class UserService {
    private readonly repo: Repository<User>;
    private readonly logger: { log: (msg: string) => void };

    constructor(repo: Repository<User>) {
        this.repo = repo;
        this.logger = { log: (msg: string) => console.log(`[UserService] ${msg}`) };
    }

    async getUser(id: string): Promise<User> {
        const user = await this.repo.findById(id);
        assertDefined(user, `User ${id} not found`);
        return user;
    }

    async listUsers(params: PaginationParams): Promise<ApiResponse<User[]>> {
        return this.repo.findAll(params);
    }

    async createUser(input: Omit<User, keyof Entity>): Promise<User> {
        if (!Validators.isEmail(input.email)) {
            throw new Error('Invalid email address');
        }
        if (!Validators.isNonEmpty(input.name)) {
            throw new Error('Name cannot be empty');
        }

        const user: User = {
            ...input,
            id: crypto.randomUUID(),
            createdAt: new Date(),
            updatedAt: new Date(),
        };

        const created = await this.repo.create(user);
        this.logger.log(`Created user ${created.id}`);
        return created;
    }

    async updateUser(id: string, updates: DeepPartial<User>): Promise<User> {
        const updated = await this.repo.update(id, updates);
        assertDefined(updated, `User ${id} not found`);
        this.logger.log(`Updated user ${id}`);
        return updated;
    }

    async deleteUser(id: string): Promise<void> {
        const deleted = await this.repo.delete(id);
        if (!deleted) {
            throw new Error(`User ${id} not found`);
        }
        this.logger.log(`Deleted user ${id}`);
    }

    async getAdmins(): Promise<Admin[]> {
        const response = await this.repo.findAll({ filters: { role: UserRole.Admin } });
        return response.data.filter(isAdmin);
    }

    async getUserPermissions(id: string): Promise<Permission[]> {
        const user = await this.getUser(id);
        if (isAdmin(user)) {
            return user.permissions;
        }
        return user.role === UserRole.Editor
            ? [Permission.Read, Permission.Write]
            : [Permission.Read];
    }
}

// ── Overloaded functions ─────────────────────────────────────────────────────

function formatValue(value: string): string;
function formatValue(value: number): string;
function formatValue(value: Date): string;
function formatValue(value: string | number | Date): string {
    if (typeof value === 'string') return value.trim();
    if (typeof value === 'number') return value.toFixed(2);
    return value.toISOString();
}

function merge<T>(target: T, source: Partial<T>): T;
function merge<T>(target: T, source: Partial<T>, deep: boolean): T;
function merge<T>(target: T, source: Partial<T>, deep?: boolean): T {
    if (deep) {
        return JSON.parse(JSON.stringify({ ...target, ...source }));
    }
    return { ...target, ...source };
}

// ── Type assertions and anti-patterns ────────────────────────────────────────

function processUser(user: User): string {
    const name = user.name as any;
    const age: number = user.age as number;
    const email = user.email!;

    if (user.email !== undefined && user.email !== null) {
        return `${name} (${email})`;
    }

    return name!;
}

function unnecessaryTypeAssertion(x: string) {
    return x as string;
}

const inferrable: number = 42;
const alsoInferrable: string = 'hello';
const boolInferrable: boolean = true;

const arr: Array<string> = ['a', 'b', 'c'];
const numArr: Array<number> = [1, 2, 3];
const record: Record<string, number> = { a: 1, b: 2 };

async function fetchUsers(url: string): Promise<User[]> {
    const response = await fetch(url);
    const data = await response.json();
    return data as User[];
}

// ── Result type pattern ──────────────────────────────────────────────────────

type Result<T, E = Error> =
    | { ok: true; value: T }
    | { ok: false; error: E };

function ok<T>(value: T): Result<T> {
    return { ok: true, value };
}

function err<E>(error: E): Result<never, E> {
    return { ok: false, error };
}

async function tryFetch<T>(url: string): Promise<Result<T>> {
    try {
        const response = await fetch(url);
        if (!response.ok) {
            return err(new Error(`HTTP ${response.status}`));
        }
        const data = await response.json();
        return ok(data as T);
    } catch (error) {
        return err(error instanceof Error ? error : new Error(String(error)));
    }
}

// ── Exports ──────────────────────────────────────────────────────────────────

export {
    Repository,
    CacheableRepository,
    UserService,
    Validators,
    formatValue,
    merge,
    tryFetch,
    ok,
    err,
    isUser,
    isAdmin,
    isNonNullable,
    assertDefined,
    UserRole,
    Permission,
    HttpStatus,
    LogLevel,
};

export type {
    Entity,
    User,
    Admin,
    UserPreferences,
    NotificationSettings,
    ApiResponse,
    ApiError,
    PaginationParams,
    Nullable,
    Optional,
    DeepPartial,
    DeepReadonly,
    Result,
};
";

/// Jest test file (~400 lines).
///
/// Simulates a realistic test suite for a user service with setup/teardown,
/// mocking, async tests, parameterized tests, error assertions, and
/// snapshot-like patterns. Exercises `CallExpression` (describe/it/expect),
/// `ArrowFunctionExpression`, nested `BlockStatement`.
pub const TEST_FIXTURE: &str = r#"
const { UserService } = require('./user-service');
const { Repository } = require('./repository');
const { validate, sanitize, formatDate } = require('./helpers');
const { createMockUser, createMockAdmin } = require('./test-utils');

// ── UserService tests ────────────────────────────────────────────────────────

describe('UserService', () => {
    let service;
    let mockRepo;
    let mockLogger;

    beforeAll(() => {
        mockLogger = {
            info: jest.fn(),
            warn: jest.fn(),
            error: jest.fn(),
        };
    });

    beforeEach(() => {
        mockRepo = {
            findById: jest.fn(),
            findAll: jest.fn(),
            create: jest.fn(),
            update: jest.fn(),
            delete: jest.fn(),
        };
        service = new UserService(mockRepo, mockLogger);
        jest.clearAllMocks();
    });

    afterEach(() => {
        jest.restoreAllMocks();
    });

    afterAll(() => {
        jest.resetModules();
    });

    describe('getUser', () => {
        it('should return a user by id', async () => {
            const mockUser = createMockUser({ id: '123', name: 'Alice' });
            mockRepo.findById.mockResolvedValue(mockUser);

            const result = await service.getUser('123');

            expect(result).toBeDefined();
            expect(result.id).toBe('123');
            expect(result.name).toBe('Alice');
            expect(mockRepo.findById).toHaveBeenCalledWith('123');
            expect(mockRepo.findById).toHaveBeenCalledTimes(1);
        });

        it('should throw when user not found', async () => {
            mockRepo.findById.mockResolvedValue(null);

            await expect(service.getUser('999')).rejects.toThrow('not found');
            expect(mockRepo.findById).toHaveBeenCalledWith('999');
        });

        it('should propagate repository errors', async () => {
            mockRepo.findById.mockRejectedValue(new Error('DB connection failed'));

            await expect(service.getUser('123')).rejects.toThrow('DB connection failed');
        });

        it.each([
            ['valid-uuid', true],
            ['', false],
            ['  ', false],
        ])('should validate id "%s" as %s', async (id, shouldSucceed) => {
            if (shouldSucceed) {
                mockRepo.findById.mockResolvedValue(createMockUser({ id }));
                const result = await service.getUser(id);
                expect(result).toBeDefined();
            } else {
                await expect(service.getUser(id)).rejects.toThrow();
            }
        });
    });

    describe('listUsers', () => {
        it('should return paginated results', async () => {
            const users = [
                createMockUser({ id: '1', name: 'Alice' }),
                createMockUser({ id: '2', name: 'Bob' }),
                createMockUser({ id: '3', name: 'Charlie' }),
            ];
            mockRepo.findAll.mockResolvedValue({
                data: users,
                meta: { page: 1, pageSize: 20, total: 3, hasMore: false },
            });

            const result = await service.listUsers({ page: 1, pageSize: 20 });

            expect(result.data).toHaveLength(3);
            expect(result.meta.total).toBe(3);
            expect(result.meta.hasMore).toBe(false);
        });

        it('should pass sort params to repository', async () => {
            mockRepo.findAll.mockResolvedValue({
                data: [],
                meta: { page: 1, pageSize: 10, total: 0, hasMore: false },
            });

            await service.listUsers({
                page: 1,
                pageSize: 10,
                sortBy: 'name',
                sortOrder: 'desc',
            });

            expect(mockRepo.findAll).toHaveBeenCalledWith(
                expect.objectContaining({
                    sortBy: 'name',
                    sortOrder: 'desc',
                })
            );
        });

        it('should handle empty results', async () => {
            mockRepo.findAll.mockResolvedValue({
                data: [],
                meta: { page: 1, pageSize: 20, total: 0, hasMore: false },
            });

            const result = await service.listUsers({});
            expect(result.data).toEqual([]);
            expect(result.meta.total).toBe(0);
        });

        it('should apply filters', async () => {
            mockRepo.findAll.mockResolvedValue({
                data: [createMockUser({ role: 'admin' })],
                meta: { page: 1, pageSize: 20, total: 1, hasMore: false },
            });

            const result = await service.listUsers({
                filters: { role: 'admin', isActive: true },
            });

            expect(result.data).toHaveLength(1);
            expect(result.data[0].role).toBe('admin');
        });
    });

    describe('createUser', () => {
        const validInput = {
            name: 'Alice Smith',
            email: 'alice@example.com',
            age: 30,
            role: 'editor',
        };

        it('should create a user with valid input', async () => {
            mockRepo.create.mockImplementation((user) => Promise.resolve(user));

            const result = await service.createUser(validInput);

            expect(result).toBeDefined();
            expect(result.name).toBe('Alice Smith');
            expect(result.email).toBe('alice@example.com');
            expect(result.id).toBeDefined();
            expect(mockRepo.create).toHaveBeenCalledTimes(1);
            expect(mockLogger.info).toHaveBeenCalled();
        });

        it('should reject invalid email', async () => {
            await expect(
                service.createUser({ ...validInput, email: 'not-an-email' })
            ).rejects.toThrow('Invalid email');

            expect(mockRepo.create).not.toHaveBeenCalled();
        });

        it('should reject empty name', async () => {
            await expect(
                service.createUser({ ...validInput, name: '' })
            ).rejects.toThrow('Name cannot be empty');
        });

        it('should trim whitespace from name', async () => {
            mockRepo.create.mockImplementation((user) => Promise.resolve(user));

            const result = await service.createUser({
                ...validInput,
                name: '  Alice Smith  ',
            });

            expect(result.name).toBe('Alice Smith');
        });

        it.each([
            ['admin', true],
            ['editor', true],
            ['viewer', true],
            ['superadmin', false],
            ['', false],
        ])('should validate role "%s" (valid: %s)', async (role, isValid) => {
            if (isValid) {
                mockRepo.create.mockImplementation((u) => Promise.resolve(u));
                const result = await service.createUser({ ...validInput, role });
                expect(result.role).toBe(role);
            } else {
                await expect(
                    service.createUser({ ...validInput, role })
                ).rejects.toThrow();
            }
        });
    });

    describe('updateUser', () => {
        it('should update existing user fields', async () => {
            const existing = createMockUser({ id: '123', name: 'Alice' });
            mockRepo.update.mockResolvedValue({ ...existing, name: 'Alice Updated' });

            const result = await service.updateUser('123', { name: 'Alice Updated' });

            expect(result.name).toBe('Alice Updated');
            expect(mockRepo.update).toHaveBeenCalledWith('123', { name: 'Alice Updated' });
        });

        it('should throw when user not found', async () => {
            mockRepo.update.mockResolvedValue(null);

            await expect(
                service.updateUser('999', { name: 'New Name' })
            ).rejects.toThrow('not found');
        });

        it('should validate email on update', async () => {
            await expect(
                service.updateUser('123', { email: 'bad-email' })
            ).rejects.toThrow('Invalid email');
        });

        it('should not allow role changes without admin permission', async () => {
            await expect(
                service.updateUser('123', { role: 'admin' }, { callerRole: 'viewer' })
            ).rejects.toThrow('Insufficient permissions');
        });
    });

    describe('deleteUser', () => {
        it('should delete existing user', async () => {
            mockRepo.delete.mockResolvedValue(true);

            await service.deleteUser('123');

            expect(mockRepo.delete).toHaveBeenCalledWith('123');
            expect(mockLogger.info).toHaveBeenCalled();
        });

        it('should throw when user not found', async () => {
            mockRepo.delete.mockResolvedValue(false);

            await expect(service.deleteUser('999')).rejects.toThrow('not found');
        });

        it('should log deletion', async () => {
            mockRepo.delete.mockResolvedValue(true);

            await service.deleteUser('123');

            expect(mockLogger.info).toHaveBeenCalledWith(
                expect.stringContaining('123')
            );
        });
    });

    describe('getAdmins', () => {
        it('should return only admin users', async () => {
            const users = [
                createMockAdmin({ id: '1', name: 'Admin1' }),
                createMockUser({ id: '2', name: 'Editor1', role: 'editor' }),
                createMockAdmin({ id: '3', name: 'Admin2' }),
            ];
            mockRepo.findAll.mockResolvedValue({
                data: users,
                meta: { page: 1, pageSize: 100, total: 3, hasMore: false },
            });

            const admins = await service.getAdmins();

            expect(admins).toHaveLength(2);
            expect(admins[0].name).toBe('Admin1');
            expect(admins[1].name).toBe('Admin2');
        });
    });
});

// ── Helper function tests ────────────────────────────────────────────────────

describe('helpers', () => {
    describe('validate', () => {
        it('should validate required fields', () => {
            const result = validate({ name: 'Alice' }, { required: ['name', 'email'] });
            expect(result.valid).toBe(false);
            expect(result.errors).toHaveLength(1);
            expect(result.errors[0].field).toBe('email');
        });

        it('should pass with all required fields', () => {
            const result = validate(
                { name: 'Alice', email: 'alice@test.com' },
                { required: ['name', 'email'] }
            );
            expect(result.valid).toBe(true);
            expect(result.errors).toHaveLength(0);
        });
    });

    describe('sanitize', () => {
        it.each([
            ['  hello  ', 'hello'],
            ['UPPER', 'upper'],
            ['  Mixed Case  ', 'mixed case'],
        ])('should sanitize "%s" to "%s"', (input, expected) => {
            expect(sanitize(input)).toBe(expected);
        });
    });

    describe('formatDate', () => {
        it('should format ISO date strings', () => {
            const result = formatDate('2024-01-15T00:00:00Z');
            expect(result).toMatch(/Jan.*15.*2024/);
        });

        it('should handle invalid dates', () => {
            expect(() => formatDate('not-a-date')).toThrow();
        });
    });
});

// ── Top-level expect (anti-pattern) ──────────────────────────────────────────

expect(1).toBe(1);

// ── Deep nesting (anti-pattern) ──────────────────────────────────────────────

describe('deeply nested', () => {
    describe('level 1', () => {
        describe('level 2', () => {
            describe('level 3', () => {
                describe('level 4', () => {
                    it('passes', () => {
                        expect(true).toBe(true);
                    });
                });
            });
        });
    });
});

// ── Async patterns ───────────────────────────────────────────────────────────

test('async with resolved promise', async () => {
    const data = await Promise.resolve({ items: [1, 2, 3] });
    expect(data.items).toBeDefined();
    expect(data.items.length).toBeGreaterThan(0);
    expect(data.items).toContain(2);
});

test('async with rejected promise', async () => {
    await expect(Promise.reject(new Error('fail'))).rejects.toThrow('fail');
});

test('test without expect', () => {
    const result = [1, 2, 3].filter(x => x > 1);
    return;
});
"#;

/// Module/import source (~350 lines).
///
/// Simulates a Node.js application module with complex import/export patterns,
/// dynamic imports, promise chains, re-exports, and async patterns.
/// Heavy on `ImportDeclaration`, `ExportDeclaration`, `CallExpression` (Promise),
/// `AwaitExpression`, `ArrowFunctionExpression`.
pub const MODULES_FIXTURE: &str = r"
// ── Imports: named, default, namespace, side-effect ──────────────────────────

import { readFile, writeFile, stat, mkdir } from 'fs/promises';
import { createReadStream, createWriteStream } from 'fs';
import path from 'path';
import * as url from 'url';
import * as crypto from 'crypto';
import { Buffer } from 'buffer';
import { createServer, IncomingMessage, ServerResponse } from 'http';
import { pipeline } from 'stream/promises';
import events from 'events';
import defaultExport from './default';
import { named1, named2, named3 } from './named';
import { foo } from './foo';
import { foo as bar } from './bar';
import { something } from './foo';
import { config as appConfig } from './config';
import type { User, Settings, ApiResponse } from './types';
import './polyfills';

// ── Re-exports ───────────────────────────────────────────────────────────────

export { named1, named2 } from './named';
export { default as utils } from './utils';
export type { User, Settings } from './types';

// ── Dynamic imports ──────────────────────────────────────────────────────────

const lazyModule = import('./lazy');

async function loadPlugin(name) {
    const pluginPath = `./plugins/${name}`;
    const plugin = await import(pluginPath);
    return plugin.default || plugin;
}

async function loadOptionalDeps() {
    let sharp;
    try {
        sharp = await import('sharp');
    } catch {
        sharp = null;
    }

    let redis;
    try {
        redis = await import('redis');
    } catch {
        redis = null;
    }

    return { sharp, redis };
}

// ── File operations ──────────────────────────────────────────────────────────

async function ensureDir(dirPath) {
    try {
        await stat(dirPath);
    } catch {
        await mkdir(dirPath, { recursive: true });
    }
}

async function readJsonFile(filePath) {
    const content = await readFile(filePath, 'utf-8');
    return JSON.parse(content);
}

async function writeJsonFile(filePath, data) {
    const dir = path.dirname(filePath);
    await ensureDir(dir);
    const json = JSON.stringify(data, null, 2);
    await writeFile(filePath, json, 'utf-8');
}

async function copyFile(src, dest) {
    const readStream = createReadStream(src);
    const writeStream = createWriteStream(dest);
    await pipeline(readStream, writeStream);
}

async function getFileHash(filePath) {
    const content = await readFile(filePath);
    return crypto.createHash('sha256').update(content).digest('hex');
}

async function walkDir(dirPath) {
    const entries = [];
    const items = await readFile(dirPath);
    for (const item of items) {
        const fullPath = path.join(dirPath, item.name);
        if (item.isDirectory()) {
            const subEntries = await walkDir(fullPath);
            entries.push(...subEntries);
        } else {
            entries.push(fullPath);
        }
    }
    return entries;
}

// ── Promise patterns ─────────────────────────────────────────────────────────

const pendingResult = new Promise((resolve, reject) => {
    setTimeout(() => {
        resolve({ status: 'ok', data: [1, 2, 3] });
    }, 1000);
});

pendingResult.then(value => {
    console.log(value);
});

const promiseChain = fetch('/api/data')
    .then(res => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        return res.json();
    })
    .then(data => data.results)
    .then(results => results.filter(r => r.active))
    .then(active => active.map(r => r.id))
    .catch(err => {
        console.error('API request failed:', err);
        return [];
    })
    .finally(() => {
        console.log('Request completed');
    });

const parallelFetch = Promise.all([
    fetch('/api/users').then(r => r.json()),
    fetch('/api/posts').then(r => r.json()),
    fetch('/api/comments').then(r => r.json()),
    fetch('/api/tags').then(r => r.json()),
]).then(([users, posts, comments, tags]) => {
    return { users, posts, comments, tags };
});

const raceFetch = Promise.race([
    fetch('/api/primary-source'),
    fetch('/api/fallback-source'),
    new Promise((_, reject) =>
        setTimeout(() => reject(new Error('Request timeout')), 5000)
    ),
]);

const allSettled = Promise.allSettled([
    fetch('/api/service-a'),
    fetch('/api/service-b'),
    fetch('/api/service-c'),
]).then(results => {
    const successes = results.filter(r => r.status === 'fulfilled');
    const failures = results.filter(r => r.status === 'rejected');
    console.log(`${successes.length} succeeded, ${failures.length} failed`);
    return successes.map(r => r.value);
});

// Nested promise (anti-pattern)
new Promise((resolve) => {
    resolve(new Promise((innerResolve) => {
        innerResolve(42);
    }));
});

// ── Async/await patterns ─────────────────────────────────────────────────────

async function fetchWithRetry(url, maxRetries) {
    let lastError;
    for (let i = 0; i < maxRetries; i++) {
        try {
            const response = await fetch(url);
            if (!response.ok) {
                throw new Error(`HTTP ${response.status}`);
            }
            return await response.json();
        } catch (error) {
            lastError = error;
            const delay = Math.pow(2, i) * 1000;
            await new Promise(resolve => setTimeout(resolve, delay));
        }
    }
    throw lastError;
}

async function processInParallel(items, processor, concurrency) {
    const results = [];
    for (let i = 0; i < items.length; i += concurrency) {
        const batch = items.slice(i, i + concurrency);
        const batchResults = await Promise.all(
            batch.map(item => processor(item))
        );
        results.push(...batchResults);
    }
    return results;
}

async function streamToBuffer(stream) {
    const chunks = [];
    for await (const chunk of stream) {
        chunks.push(chunk);
    }
    return Buffer.concat(chunks);
}

// async without await (anti-pattern)
async function noAwait() {
    return 42;
}

// ── HTTP server ──────────────────────────────────────────────────────────────

function createApiServer(routes) {
    return createServer(async (req, res) => {
        const parsedUrl = new url.URL(req.url, `http://${req.headers.host}`);
        const pathname = parsedUrl.pathname;
        const handler = routes[pathname];

        if (!handler) {
            res.writeHead(404);
            res.end(JSON.stringify({ error: 'Not found' }));
            return;
        }

        try {
            const body = await streamToBuffer(req);
            const parsed = body.length > 0 ? JSON.parse(body.toString()) : {};
            const result = await handler(parsed, parsedUrl.searchParams);

            res.writeHead(200, { 'Content-Type': 'application/json' });
            res.end(JSON.stringify(result));
        } catch (error) {
            res.writeHead(500);
            res.end(JSON.stringify({ error: error.message }));
        }
    });
}

// ── Path utilities ───────────────────────────────────────────────────────────

const dataDir = path.join(process.cwd(), 'data');
const outputDir = path.join(dataDir, 'output');
const tempDir = path.join(dataDir, 'temp');
const configPath = path.resolve('config.json');
const logDir = path.join(process.cwd(), 'logs');

function resolveRelative(basePath, relativePath) {
    return path.resolve(path.dirname(basePath), relativePath);
}

function getExtension(filePath) {
    return path.extname(filePath).toLowerCase();
}

function changeExtension(filePath, newExt) {
    const dir = path.dirname(filePath);
    const base = path.basename(filePath, path.extname(filePath));
    return path.join(dir, `${base}${newExt}`);
}

// ── Exports ──────────────────────────────────────────────────────────────────

export {
    readJsonFile,
    writeJsonFile,
    copyFile,
    getFileHash,
    walkDir,
    ensureDir,
    fetchWithRetry,
    processInParallel,
    streamToBuffer,
    createApiServer,
    loadPlugin,
    loadOptionalDeps,
    resolveRelative,
    getExtension,
    changeExtension,
};

export default readJsonFile;
";

/// Vue component source (~250 lines).
///
/// Simulates a realistic Vue 3 composition API component with refs, reactives,
/// computed properties, watchers, lifecycle hooks, and emit patterns.
/// Exercises `CallExpression` (Vue APIs), `ObjectExpression`, `ArrowFunctionExpression`.
pub const VUE_FIXTURE: &str = r"
import {
    defineComponent,
    ref,
    reactive,
    computed,
    watch,
    watchEffect,
    onMounted,
    onBeforeUnmount,
    toRefs,
    provide,
    inject,
    nextTick,
} from 'vue';

// ── Composable: useUsers ─────────────────────────────────────────────────────

function useUsers(initialFilters) {
    const users = ref([]);
    const loading = ref(false);
    const error = ref(null);
    const filters = reactive({
        search: '',
        role: '',
        sortBy: 'name',
        sortOrder: 'asc',
        ...initialFilters,
    });

    const filteredUsers = computed(() => {
        let result = users.value;

        if (filters.search) {
            const query = filters.search.toLowerCase();
            result = result.filter(
                user =>
                    user.name.toLowerCase().includes(query) ||
                    user.email.toLowerCase().includes(query)
            );
        }

        if (filters.role) {
            result = result.filter(user => user.role === filters.role);
        }

        result = [...result].sort((a, b) => {
            const aVal = a[filters.sortBy] || '';
            const bVal = b[filters.sortBy] || '';
            const cmp = String(aVal).localeCompare(String(bVal));
            return filters.sortOrder === 'desc' ? -cmp : cmp;
        });

        return result;
    });

    const totalCount = computed(() => users.value.length);
    const filteredCount = computed(() => filteredUsers.value.length);

    async function fetchUsers() {
        loading.value = true;
        error.value = null;
        try {
            const response = await fetch('/api/users');
            if (!response.ok) {
                throw new Error(`HTTP ${response.status}`);
            }
            users.value = await response.json();
        } catch (err) {
            error.value = err.message;
            users.value = [];
        } finally {
            loading.value = false;
        }
    }

    async function deleteUser(id) {
        try {
            await fetch(`/api/users/${id}`, { method: 'DELETE' });
            users.value = users.value.filter(u => u.id !== id);
        } catch (err) {
            error.value = `Failed to delete user: ${err.message}`;
        }
    }

    async function updateUser(id, updates) {
        try {
            const response = await fetch(`/api/users/${id}`, {
                method: 'PATCH',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(updates),
            });
            const updated = await response.json();
            const index = users.value.findIndex(u => u.id === id);
            if (index >= 0) {
                users.value[index] = { ...users.value[index], ...updated };
            }
        } catch (err) {
            error.value = `Failed to update user: ${err.message}`;
        }
    }

    return {
        users,
        loading,
        error,
        filters,
        filteredUsers,
        totalCount,
        filteredCount,
        fetchUsers,
        deleteUser,
        updateUser,
    };
}

// ── Composable: usePagination ────────────────────────────────────────────────

function usePagination(items, pageSize) {
    const currentPage = ref(1);
    const size = ref(pageSize || 20);

    const totalPages = computed(() =>
        Math.ceil(items.value.length / size.value)
    );

    const paginatedItems = computed(() => {
        const start = (currentPage.value - 1) * size.value;
        return items.value.slice(start, start + size.value);
    });

    const hasNext = computed(() => currentPage.value < totalPages.value);
    const hasPrev = computed(() => currentPage.value > 1);

    function nextPage() {
        if (hasNext.value) {
            currentPage.value += 1;
        }
    }

    function prevPage() {
        if (hasPrev.value) {
            currentPage.value -= 1;
        }
    }

    function goToPage(page) {
        const clamped = Math.max(1, Math.min(page, totalPages.value));
        currentPage.value = clamped;
    }

    watch(items, () => {
        if (currentPage.value > totalPages.value) {
            currentPage.value = Math.max(1, totalPages.value);
        }
    });

    return {
        currentPage,
        totalPages,
        paginatedItems,
        hasNext,
        hasPrev,
        nextPage,
        prevPage,
        goToPage,
    };
}

// ── Main component ───────────────────────────────────────────────────────────

export default defineComponent({
    name: 'UserManagement',

    props: {
        title: {
            type: String,
            required: true,
        },
        maxItems: {
            type: Number,
            default: 100,
        },
        showFilters: {
            type: Boolean,
            default: true,
        },
        initialRole: {
            type: String,
            default: '',
        },
    },

    emits: ['user-selected', 'user-deleted', 'error'],

    setup(props, { emit }) {
        const {
            users,
            loading,
            error,
            filters,
            filteredUsers,
            totalCount,
            filteredCount,
            fetchUsers,
            deleteUser,
            updateUser,
        } = useUsers({ role: props.initialRole });

        const { paginatedItems, currentPage, totalPages, hasNext, hasPrev, nextPage, prevPage } =
            usePagination(filteredUsers, 20);

        const selectedUserId = ref(null);
        const showConfirmDialog = ref(false);
        const pendingDeleteId = ref(null);

        const selectedUser = computed(() => {
            if (!selectedUserId.value) return null;
            return users.value.find(u => u.id === selectedUserId.value) || null;
        });

        function selectUser(user) {
            selectedUserId.value = user.id;
            emit('user-selected', user);
        }

        function confirmDelete(id) {
            pendingDeleteId.value = id;
            showConfirmDialog.value = true;
        }

        async function executeDelete() {
            if (pendingDeleteId.value) {
                await deleteUser(pendingDeleteId.value);
                emit('user-deleted', pendingDeleteId.value);
                showConfirmDialog.value = false;
                pendingDeleteId.value = null;
            }
        }

        watch(error, (newError) => {
            if (newError) {
                emit('error', newError);
            }
        });

        watchEffect(() => {
            console.log(
                `Showing ${filteredCount.value} of ${totalCount.value} users ` +
                `(page ${currentPage.value}/${totalPages.value})`
            );
        });

        onMounted(async () => {
            await fetchUsers();
            await nextTick();
            console.log('UserManagement mounted with', totalCount.value, 'users');
        });

        let pollInterval = null;
        onMounted(() => {
            pollInterval = setInterval(() => {
                fetchUsers();
            }, 30000);
        });

        onBeforeUnmount(() => {
            if (pollInterval) {
                clearInterval(pollInterval);
            }
        });

        provide('userService', { fetchUsers, deleteUser, updateUser });

        return {
            users,
            loading,
            error,
            filters,
            filteredUsers,
            paginatedItems,
            currentPage,
            totalPages,
            hasNext,
            hasPrev,
            nextPage,
            prevPage,
            selectedUser,
            showConfirmDialog,
            totalCount,
            filteredCount,
            selectUser,
            confirmDelete,
            executeDelete,
        };
    },
});
";

/// `JSDoc` source (~300 lines).
///
/// Simulates a well-documented utility library with `JSDoc` comments on
/// functions, classes, typedefs, and methods. Mixes documented and
/// undocumented functions. Exercises comment parsing and JSDoc-specific rules.
pub const JSDOC_FIXTURE: &str = r"
/**
 * @module data-utils
 * @description Comprehensive data utility library for transformations,
 * validation, and formatting.
 * @version 2.4.0
 * @author Engineering Team
 */

// ── Math utilities ───────────────────────────────────────────────────────────

/**
 * Add two numbers together.
 * @param {number} a - First operand
 * @param {number} b - Second operand
 * @returns {number} The sum of a and b
 * @example
 * add(2, 3); // 5
 */
function add(a, b) {
    return a + b;
}

/**
 * Subtract b from a.
 * @param {number} a - The minuend
 * @param {number} b - The subtrahend
 * @returns {number} The difference
 */
function subtract(a, b) {
    return a - b;
}

/**
 * Multiply two numbers.
 * @param {number} a
 * @param {number} b
 * @returns {number}
 */
function multiply(a, b) {
    return a * b;
}

/**
 * Safely divide a by b.
 * @param {number} a - Dividend
 * @param {number} b - Divisor (must not be zero)
 * @returns {number} The quotient
 * @throws {Error} When divisor is zero
 */
function divide(a, b) {
    if (b === 0) {
        throw new Error('Division by zero');
    }
    return a / b;
}

/**
 * Clamp a value between min and max.
 * @param {number} value - The value to clamp
 * @param {number} min - Minimum boundary
 * @param {number} max - Maximum boundary
 * @returns {number} The clamped value
 */
function clamp(value, min, max) {
    return Math.max(min, Math.min(max, value));
}

/**
 * Calculate the average of an array of numbers.
 * @param {number[]} numbers - Array of numbers
 * @returns {number} The arithmetic mean
 */
function average(numbers) {
    if (numbers.length === 0) return 0;
    const sum = numbers.reduce((acc, n) => acc + n, 0);
    return sum / numbers.length;
}

/**
 * Round a number to a specified number of decimal places.
 * @param {number} value - The number to round
 * @param {number} [decimals=2] - Number of decimal places
 * @returns {number} The rounded number
 */
function round(value, decimals) {
    const factor = Math.pow(10, decimals || 2);
    return Math.round(value * factor) / factor;
}

// ── String utilities ─────────────────────────────────────────────────────────

/**
 * Capitalize the first letter of a string.
 * @param {string} str - Input string
 * @returns {string} Capitalized string
 */
function capitalize(str) {
    if (!str) return '';
    return str.charAt(0).toUpperCase() + str.slice(1);
}

/**
 * Convert a string to kebab-case.
 * @param {string} str - Input string (camelCase or PascalCase)
 * @returns {string} kebab-case string
 */
function toKebabCase(str) {
    return str
        .replace(/([a-z])([A-Z])/g, '$1-$2')
        .replace(/\s+/g, '-')
        .toLowerCase();
}

/**
 * Truncate a string to a maximum length.
 * @param {string} str - String to truncate
 * @param {number} maxLength - Maximum length
 * @param {string} [suffix='...'] - Suffix to append
 * @returns {string} Truncated string
 */
function truncate(str, maxLength, suffix) {
    if (str.length <= maxLength) return str;
    const s = suffix || '...';
    return str.slice(0, maxLength - s.length) + s;
}

/**
 * @param {string} template - Template string with {key} placeholders
 * @param {Object.<string, string>} values - Key-value pairs for substitution
 * @returns {string} Interpolated string
 */
function interpolate(template, values) {
    return template.replace(/\{(\w+)\}/g, (_, key) => {
        return values[key] !== undefined ? String(values[key]) : `{${key}}`;
    });
}

// Undocumented function (triggers jsdoc rules)
function slugify(str) {
    return str
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, '-')
        .replace(/^-|-$/g, '');
}

// ── Validation ───────────────────────────────────────────────────────────────

/**
 * @typedef {Object} ValidationResult
 * @property {boolean} valid - Whether validation passed
 * @property {string[]} errors - List of error messages
 */

/**
 * @typedef {Object} ValidationRule
 * @property {string} field - Field name to validate
 * @property {string} type - Expected type ('string', 'number', 'boolean')
 * @property {boolean} [required] - Whether the field is required
 * @property {number} [minLength] - Minimum string length
 * @property {number} [maxLength] - Maximum string length
 * @property {number} [min] - Minimum numeric value
 * @property {number} [max] - Maximum numeric value
 * @property {RegExp} [pattern] - Pattern to match
 */

/**
 * Validate an object against a set of rules.
 * @param {Object} data - The object to validate
 * @param {ValidationRule[]} rules - Validation rules
 * @returns {ValidationResult} The validation result
 */
function validateObject(data, rules) {
    const errors = [];

    for (const rule of rules) {
        const value = data[rule.field];

        if (rule.required && (value === undefined || value === null || value === '')) {
            errors.push(`${rule.field} is required`);
            continue;
        }

        if (value === undefined || value === null) continue;

        if (rule.type && typeof value !== rule.type) {
            errors.push(`${rule.field} must be of type ${rule.type}`);
        }

        if (rule.minLength && typeof value === 'string' && value.length < rule.minLength) {
            errors.push(`${rule.field} must be at least ${rule.minLength} characters`);
        }

        if (rule.maxLength && typeof value === 'string' && value.length > rule.maxLength) {
            errors.push(`${rule.field} must be at most ${rule.maxLength} characters`);
        }

        if (rule.min !== undefined && typeof value === 'number' && value < rule.min) {
            errors.push(`${rule.field} must be at least ${rule.min}`);
        }

        if (rule.max !== undefined && typeof value === 'number' && value > rule.max) {
            errors.push(`${rule.field} must be at most ${rule.max}`);
        }

        if (rule.pattern && typeof value === 'string' && !rule.pattern.test(value)) {
            errors.push(`${rule.field} does not match the expected pattern`);
        }
    }

    return { valid: errors.length === 0, errors };
}

// ── Data formatting ──────────────────────────────────────────────────────────

/**
 * @deprecated Use `Intl.NumberFormat` instead
 * @param {number} value - Numeric value
 * @param {string} [currency='USD'] - Currency code
 * @returns {string} Formatted currency string
 */
function formatCurrency(value, currency) {
    const c = currency || 'USD';
    return `${c} ${value.toFixed(2)}`;
}

/**
 * Format a date object to a localized string.
 * @param {Date} date - The date to format
 * @param {string} [locale='en-US'] - Locale string
 * @returns {string} Formatted date string
 */
function formatDate(date, locale) {
    return date.toLocaleDateString(locale || 'en-US', {
        year: 'numeric',
        month: 'long',
        day: 'numeric',
    });
}

// ── Class with JSDoc ─────────────────────────────────────────────────────────

/**
 * A generic collection class with common array operations.
 * @template T
 */
class Collection {
    /**
     * Create a new Collection.
     * @param {T[]} [items=[]] - Initial items
     */
    constructor(items) {
        /** @type {T[]} */
        this.items = items || [];
    }

    /**
     * Add an item to the collection.
     * @param {T} item - Item to add
     * @returns {Collection<T>} This collection (for chaining)
     */
    add(item) {
        this.items.push(item);
        return this;
    }

    /**
     * Remove an item by predicate.
     * @param {function(T): boolean} predicate - Test function
     * @returns {T|undefined} The removed item, if found
     */
    remove(predicate) {
        const index = this.items.findIndex(predicate);
        if (index >= 0) {
            return this.items.splice(index, 1)[0];
        }
        return undefined;
    }

    /**
     * Find items matching a predicate.
     * @param {function(T): boolean} predicate - Test function
     * @returns {T[]} Matching items
     */
    filter(predicate) {
        return this.items.filter(predicate);
    }

    /**
     * Transform all items.
     * @template U
     * @param {function(T): U} mapper - Transform function
     * @returns {U[]} Transformed items
     */
    map(mapper) {
        return this.items.map(mapper);
    }

    // Undocumented method (triggers jsdoc rules)
    reduce(fn, initial) {
        return this.items.reduce(fn, initial);
    }

    // Undocumented method
    forEach(fn) {
        this.items.forEach(fn);
    }

    /**
     * Get the number of items.
     * @returns {number} Item count
     */
    get size() {
        return this.items.length;
    }

    /**
     * Check if collection is empty.
     * @returns {boolean} True if empty
     */
    isEmpty() {
        return this.items.length === 0;
    }
}

// Undocumented functions (trigger jsdoc rules)
function compose(...fns) {
    return (x) => fns.reduceRight((acc, fn) => fn(acc), x);
}

function pipe(...fns) {
    return (x) => fns.reduce((acc, fn) => fn(acc), x);
}

function identity(x) {
    return x;
}

function constant(x) {
    return () => x;
}

export {
    add, subtract, multiply, divide, clamp, average, round,
    capitalize, toKebabCase, truncate, interpolate, slugify,
    validateObject, formatCurrency, formatDate,
    Collection, compose, pipe, identity, constant,
};
";

/// Storybook CSF source (~250 lines).
///
/// Simulates a realistic storybook file with multiple stories, decorators,
/// play functions, and component documentation. Uses CSF 3.0 format.
/// Exercises `ExportDeclaration`, `ObjectExpression`, `JSXElement`.
pub const STORYBOOK_FIXTURE: &str = r#"
import type { Meta, StoryObj } from '@storybook/react';
import { within, userEvent, expect } from '@storybook/test';
import { fn } from '@storybook/test';
import { Button } from './Button';
import { ThemeProvider } from './ThemeProvider';
import { IconSearch, IconPlus, IconTrash, IconEdit } from './Icons';

// ── Meta configuration ───────────────────────────────────────────────────────

const meta: Meta<typeof Button> = {
    title: 'Components/Button',
    component: Button,
    tags: ['autodocs'],
    argTypes: {
        variant: {
            control: { type: 'select' },
            options: ['primary', 'secondary', 'danger', 'ghost', 'link'],
            description: 'Visual style variant',
            table: {
                type: { summary: 'string' },
                defaultValue: { summary: 'primary' },
            },
        },
        size: {
            control: { type: 'radio' },
            options: ['xs', 'sm', 'md', 'lg', 'xl'],
            description: 'Button size',
        },
        disabled: {
            control: 'boolean',
            description: 'Disable the button',
        },
        loading: {
            control: 'boolean',
            description: 'Show loading spinner',
        },
        fullWidth: {
            control: 'boolean',
            description: 'Stretch to container width',
        },
        onClick: {
            action: 'clicked',
            description: 'Click event handler',
        },
    },
    args: {
        onClick: fn(),
        disabled: false,
        loading: false,
        fullWidth: false,
    },
    parameters: {
        layout: 'centered',
        docs: {
            description: {
                component: 'A versatile button component supporting multiple variants, sizes, and states.',
            },
        },
    },
    decorators: [
        (Story) => (
            <ThemeProvider>
                <div style={{ padding: '2rem' }}>
                    <Story />
                </div>
            </ThemeProvider>
        ),
    ],
};

export default meta;
type Story = StoryObj<typeof Button>;

// ── Basic variants ───────────────────────────────────────────────────────────

export const Primary: Story = {
    args: {
        variant: 'primary',
        children: 'Primary Button',
    },
};

export const Secondary: Story = {
    args: {
        variant: 'secondary',
        children: 'Secondary Button',
    },
};

export const Danger: Story = {
    args: {
        variant: 'danger',
        children: 'Delete',
    },
};

export const Ghost: Story = {
    args: {
        variant: 'ghost',
        children: 'Ghost Button',
    },
};

export const LinkButton: Story = {
    args: {
        variant: 'link',
        children: 'Link Style',
    },
};

// ── Sizes ────────────────────────────────────────────────────────────────────

export const ExtraSmall: Story = {
    args: { size: 'xs', children: 'Extra Small' },
};

export const Small: Story = {
    args: { size: 'sm', children: 'Small' },
};

export const Medium: Story = {
    args: { size: 'md', children: 'Medium' },
};

export const Large: Story = {
    args: { size: 'lg', children: 'Large' },
};

export const ExtraLarge: Story = {
    args: { size: 'xl', children: 'Extra Large' },
};

// ── States ───────────────────────────────────────────────────────────────────

export const Disabled: Story = {
    args: {
        disabled: true,
        children: 'Disabled',
    },
};

export const Loading: Story = {
    args: {
        loading: true,
        children: 'Saving...',
    },
};

export const FullWidth: Story = {
    args: {
        fullWidth: true,
        children: 'Full Width Button',
    },
    decorators: [
        (Story) => (
            <div style={{ width: '400px' }}>
                <Story />
            </div>
        ),
    ],
};

// ── With icons ───────────────────────────────────────────────────────────────

export const WithLeadingIcon: Story = {
    args: {
        children: 'Search',
    },
    render: (args) => (
        <Button {...args}>
            <IconSearch size={16} />
            <span>{args.children}</span>
        </Button>
    ),
};

export const WithTrailingIcon: Story = {
    args: {
        children: 'Add Item',
    },
    render: (args) => (
        <Button {...args}>
            <span>{args.children}</span>
            <IconPlus size={16} />
        </Button>
    ),
};

export const IconOnly: Story = {
    args: {
        'aria-label': 'Delete item',
        variant: 'danger',
        size: 'sm',
    },
    render: (args) => (
        <Button {...args}>
            <IconTrash size={16} />
        </Button>
    ),
};

// ── Button group ─────────────────────────────────────────────────────────────

export const ButtonGroup: Story = {
    render: () => (
        <div style={{ display: 'flex', gap: '8px' }}>
            <Button variant="primary">Save</Button>
            <Button variant="secondary">Cancel</Button>
            <Button variant="danger">Delete</Button>
        </div>
    ),
};

// ── Interactive tests (play functions) ───────────────────────────────────────

export const ClickInteraction: Story = {
    args: {
        children: 'Click Me',
    },
    play: async ({ canvasElement, args }) => {
        const canvas = within(canvasElement);
        const button = canvas.getByRole('button');

        await userEvent.click(button);
        await expect(args.onClick).toHaveBeenCalledTimes(1);
    },
};

export const DisabledNoClick: Story = {
    args: {
        children: 'Disabled',
        disabled: true,
    },
    play: async ({ canvasElement, args }) => {
        const canvas = within(canvasElement);
        const button = canvas.getByRole('button');

        await expect(button).toBeDisabled();
        await userEvent.click(button);
        await expect(args.onClick).not.toHaveBeenCalled();
    },
};

export const KeyboardNavigation: Story = {
    args: {
        children: 'Focus Me',
    },
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        const button = canvas.getByRole('button');

        await userEvent.tab();
        await expect(button).toHaveFocus();

        await userEvent.keyboard('{Enter}');
    },
};

// ── All variants showcase ────────────────────────────────────────────────────

export const AllVariants: Story = {
    render: () => (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
            {(['primary', 'secondary', 'danger', 'ghost', 'link']).map((variant) => (
                <div key={variant} style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
                    <span style={{ width: '80px', fontSize: '12px' }}>{variant}:</span>
                    {(['xs', 'sm', 'md', 'lg', 'xl']).map((size) => (
                        <Button key={`${variant}-${size}`} variant={variant} size={size}>
                            {size}
                        </Button>
                    ))}
                </div>
            ))}
        </div>
    ),
};
"#;

/// Next.js page source (~350 lines).
///
/// Simulates a realistic Next.js page with SSR, multiple components,
/// Image/Link/Head/Script usage, and common Next.js patterns.
/// Heavy on `JSXOpeningElement` (Next.js-specific elements), `JSXAttribute`.
pub const NEXTJS_FIXTURE: &str = r#"
import React, { useState, useEffect, useCallback } from 'react';
import Link from 'next/link';
import Image from 'next/image';
import Head from 'next/head';
import Script from 'next/script';
import { useRouter } from 'next/router';

// ── Types ────────────────────────────────────────────────────────────────────

interface Post {
    id: number;
    title: string;
    excerpt: string;
    image: string;
    author: {
        name: string;
        avatar: string;
    };
    publishedAt: string;
    tags: string[];
    slug: string;
}

interface PageProps {
    posts: Post[];
    totalPages: number;
    currentPage: number;
    featuredPost: Post | null;
}

// ── Components ───────────────────────────────────────────────────────────────

function PostCard({ post }: { post: Post }) {
    return (
        <article className="post-card">
            <div className="post-image">
                <Image
                    src={post.image}
                    alt={`Cover image for ${post.title}`}
                    width={400}
                    height={250}
                    priority={false}
                />
            </div>
            <div className="post-content">
                <div className="post-tags">
                    {post.tags.map((tag) => (
                        <Link key={tag} href={`/tags/${tag}`}>
                            <span className="tag">{tag}</span>
                        </Link>
                    ))}
                </div>
                <h2>
                    <Link href={`/posts/${post.slug}`}>
                        {post.title}
                    </Link>
                </h2>
                <p className="excerpt">{post.excerpt}</p>
                <div className="post-meta">
                    <Image
                        src={post.author.avatar}
                        alt={`${post.author.name} avatar`}
                        width={32}
                        height={32}
                        className="avatar"
                    />
                    <span className="author-name">{post.author.name}</span>
                    <time dateTime={post.publishedAt}>
                        {new Date(post.publishedAt).toLocaleDateString()}
                    </time>
                </div>
            </div>
        </article>
    );
}

function FeaturedPost({ post }: { post: Post }) {
    return (
        <section className="featured" aria-label="Featured post">
            <div className="featured-image">
                <Image
                    src={post.image}
                    alt={`Featured: ${post.title}`}
                    width={1200}
                    height={600}
                    priority={true}
                />
            </div>
            <div className="featured-content">
                <span className="badge">Featured</span>
                <h1>
                    <Link href={`/posts/${post.slug}`}>
                        {post.title}
                    </Link>
                </h1>
                <p>{post.excerpt}</p>
                <div className="featured-meta">
                    <Image
                        src={post.author.avatar}
                        alt={post.author.name}
                        width={48}
                        height={48}
                    />
                    <div>
                        <strong>{post.author.name}</strong>
                        <time dateTime={post.publishedAt}>
                            {new Date(post.publishedAt).toLocaleDateString('en-US', {
                                year: 'numeric',
                                month: 'long',
                                day: 'numeric',
                            })}
                        </time>
                    </div>
                </div>
            </div>
        </section>
    );
}

function Pagination({
    currentPage,
    totalPages,
}: {
    currentPage: number;
    totalPages: number;
}) {
    const pages = Array.from({ length: totalPages }, (_, i) => i + 1);

    return (
        <nav aria-label="Blog pagination" className="pagination">
            {currentPage > 1 && (
                <Link href={`/blog?page=${currentPage - 1}`}>
                    Previous
                </Link>
            )}
            {pages.map((page) => (
                <Link
                    key={page}
                    href={`/blog?page=${page}`}
                    className={page === currentPage ? 'active' : ''}
                    aria-current={page === currentPage ? 'page' : undefined}
                >
                    {page}
                </Link>
            ))}
            {currentPage < totalPages && (
                <Link href={`/blog?page=${currentPage + 1}`}>
                    Next
                </Link>
            )}
        </nav>
    );
}

function NewsletterSignup() {
    const [email, setEmail] = useState('');
    const [submitted, setSubmitted] = useState(false);

    const handleSubmit = useCallback(async (e: React.FormEvent) => {
        e.preventDefault();
        try {
            await fetch('/api/newsletter', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ email }),
            });
            setSubmitted(true);
        } catch (err) {
            console.error('Newsletter signup failed:', err);
        }
    }, [email]);

    if (submitted) {
        return <p className="success">Thanks for subscribing!</p>;
    }

    return (
        <form onSubmit={handleSubmit} className="newsletter">
            <label htmlFor="newsletter-email">Subscribe to our newsletter</label>
            <div className="input-group">
                <input
                    id="newsletter-email"
                    type="email"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    placeholder="your@email.com"
                    required
                />
                <button type="submit">Subscribe</button>
            </div>
        </form>
    );
}

// ── Main page ────────────────────────────────────────────────────────────────

function BlogPage({ posts, totalPages, currentPage, featuredPost }: PageProps) {
    const router = useRouter();

    useEffect(() => {
        document.title = `Blog - Page ${currentPage}`;
    }, [currentPage]);

    return (
        <div className="blog-page">
            <Head>
                <title>Blog - My Site</title>
                <meta name="description" content="Read our latest blog posts" />
                <meta property="og:title" content="Blog - My Site" />
                <meta property="og:type" content="website" />
                <link rel="canonical" href={`https://example.com/blog?page=${currentPage}`} />
            </Head>

            <Script
                src="https://www.googletagmanager.com/gtag/js?id=GA_ID"
                strategy="afterInteractive"
            />

            <header className="blog-header">
                <nav aria-label="Main navigation">
                    <Link href="/">Home</Link>
                    <Link href="/blog">Blog</Link>
                    <Link href="/about">About</Link>
                    <Link href="/contact">Contact</Link>
                </nav>
                <h1>Blog</h1>
            </header>

            {/* Anti-patterns for nextjs rules */}
            <img src="/hero-banner.png" alt="Blog hero" width={1200} height={400} />
            <a href="/about">About (should use Link)</a>
            <script src="https://cdn.example.com/analytics.js"></script>

            <main>
                {featuredPost && <FeaturedPost post={featuredPost} />}

                <section className="post-grid" aria-label="Blog posts">
                    {posts.map((post) => (
                        <PostCard key={post.id} post={post} />
                    ))}
                </section>

                {posts.length === 0 && (
                    <div className="empty-state">
                        <p>No posts found.</p>
                        <Link href="/blog">View all posts</Link>
                    </div>
                )}

                <Pagination currentPage={currentPage} totalPages={totalPages} />
            </main>

            <aside className="sidebar">
                <NewsletterSignup />
                <div className="popular-tags">
                    <h3>Popular Tags</h3>
                    <ul>
                        {['react', 'nextjs', 'typescript', 'css', 'node'].map((tag) => (
                            <li key={tag}>
                                <Link href={`/tags/${tag}`}>{tag}</Link>
                            </li>
                        ))}
                    </ul>
                </div>
            </aside>

            <footer className="blog-footer">
                <p>Copyright 2024</p>
                <nav>
                    <Link href="/privacy">Privacy Policy</Link>
                    <Link href="/terms">Terms of Service</Link>
                </nav>
            </footer>
        </div>
    );
}

// ── SSR ──────────────────────────────────────────────────────────────────────

export async function getServerSideProps(context) {
    const page = parseInt(context.query.page || '1', 10);
    const pageSize = 12;

    try {
        const [postsRes, featuredRes] = await Promise.all([
            fetch(`https://api.example.com/posts?page=${page}&size=${pageSize}`),
            fetch('https://api.example.com/posts/featured'),
        ]);

        const postsData = await postsRes.json();
        const featuredData = await featuredRes.json();

        return {
            props: {
                posts: postsData.items,
                totalPages: Math.ceil(postsData.total / pageSize),
                currentPage: page,
                featuredPost: featuredData || null,
            },
        };
    } catch (error) {
        return {
            props: {
                posts: [],
                totalPages: 0,
                currentPage: 1,
                featuredPost: null,
            },
        };
    }
}

export default BlogPage;
"#;
