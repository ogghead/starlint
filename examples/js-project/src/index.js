/**
 * A simple utility module that passes all starlint rules.
 */

const GREETING = "Hello from starlint-example!";

function greet(name) {
  return `${GREETING} Welcome, ${name}!`;
}

function add(a, b) {
  return a + b;
}

module.exports = { greet, add };
