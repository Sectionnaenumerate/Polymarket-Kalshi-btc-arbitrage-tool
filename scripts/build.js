#!/usr/bin/env node
'use strict';

const fs = require('fs');
const path = require('path');

console.log('Building Polymarket-Kalshi-btc-arbitrage-tool...');
console.log('Compiling source files...');

const src = path.join(__dirname, '..', 'src');
const dist = path.join(__dirname, '..', 'dist');

if (!fs.existsSync(dist)) fs.mkdirSync(dist, { recursive: true });

const files = fs.readdirSync(src).filter(f => f.endsWith('.js'));
files.forEach(f => {
    fs.copyFileSync(path.join(src, f), path.join(dist, f));
    console.log('  ✓ ' + f);
});

console.log('Build complete. ' + files.length + ' files processed.');
