import init, { parse_markdown_js } from '../markdown_engine/pkg/markdown_engine.js';
import fs from 'fs';

await init();

const content = fs.readFileSync('./public/demo.md', 'utf8');
const ast = parse_markdown_js(content);

console.log(JSON.stringify(ast, null, 2));
// Also inspect the first block directly
console.log("First block:", ast.blocks[0]);
console.log("Is it a Map?", ast.blocks[0] instanceof Map);
console.log("Type of block:", typeof ast.blocks[0]);
console.log("Keys:", Object.keys(ast.blocks[0]));
