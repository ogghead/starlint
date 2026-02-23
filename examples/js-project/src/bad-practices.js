/**
 * This file contains intentional lint errors to demonstrate starlint's
 * correctness rules. Run `npm run lint` to see the diagnostics.
 */

function processInput(input) {
  // starlint: no-debugger (error)
  debugger;

  // starlint: no-eval (error)
  const result = eval("1 + 2");

  return result;
}

module.exports = { processInput };
