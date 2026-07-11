type ExpressionToken =
  | { type: 'number'; value: number }
  | { type: 'operator'; value: string }
  | { type: 'paren'; value: '(' | ')' }

export function evaluateExpressionLine(lineText: string) {
  const expression = extractExpression(lineText)
  if (!expression) return null

  try {
    const tokens = tokenizeExpression(expression)
    if (tokens.length === 0) return null
    const result = evaluateTokens(tokens)
    return Number.isFinite(result) ? result : null
  } catch {
    return null
  }
}

export function formatCalculationResult(result: number) {
  const rounded = Math.abs(result) < 1e-12 ? 0 : result
  return Number.isInteger(rounded) ? String(rounded) : String(Number(rounded.toPrecision(12)))
}

function extractExpression(lineText: string) {
  const normalized = lineText.replace(/×/g, '*').replace(/÷/g, '/').replace(/，/g, ',')
  const start = normalized.search(/[-+.\d(]/)
  if (start === -1) return ''

  let expression = ''
  for (let index = start; index < normalized.length; index += 1) {
    const char = normalized[index]
    if (/[\d+\-*/%^().\s]/.test(char)) expression += char
    else break
  }

  expression = expression.trim().replace(/[+\-*/%^.\s]+$/g, '').trim()
  while (unmatchedOpenParens(expression) > 0) expression += ')'
  return expression
}

function unmatchedOpenParens(expression: string) {
  let depth = 0
  for (const char of expression) {
    if (char === '(') depth += 1
    else if (char === ')') depth -= 1
  }
  return Math.max(0, depth)
}

function tokenizeExpression(expression: string) {
  const tokens: ExpressionToken[] = []
  let index = 0

  while (index < expression.length) {
    const char = expression[index]
    if (/\s/.test(char)) {
      index += 1
      continue
    }
    if (char === '(' || char === ')') {
      tokens.push({ type: 'paren', value: char })
      index += 1
      continue
    }
    if (/[+\-*/%^]/.test(char)) {
      const previous = tokens[tokens.length - 1]
      const unary = (char === '+' || char === '-')
        && (!previous || previous.type === 'operator' || (previous.type === 'paren' && previous.value === '('))
      if (unary && /[\d.]/.test(expression[index + 1] ?? '')) {
        const match = expression.slice(index).match(/^[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:e[+-]?\d+)?/i)
        if (!match) throw new Error('invalid number')
        tokens.push({ type: 'number', value: Number(match[0]) })
        index += match[0].length
        continue
      }
      tokens.push({ type: 'operator', value: char })
      index += 1
      continue
    }

    const match = expression.slice(index).match(/^(?:\d+(?:\.\d*)?|\.\d+)(?:e[+-]?\d+)?/i)
    if (!match) throw new Error('invalid token')
    tokens.push({ type: 'number', value: Number(match[0]) })
    index += match[0].length
  }
  return tokens
}

function evaluateTokens(tokens: ExpressionToken[]) {
  const output: ExpressionToken[] = []
  const operators: ExpressionToken[] = []
  const precedence: Record<string, number> = { '+': 1, '-': 1, '*': 2, '/': 2, '%': 2, '^': 3 }

  for (const token of tokens) {
    if (token.type === 'number') {
      output.push(token)
    } else if (token.type === 'paren') {
      if (token.value === '(') operators.push(token)
      else {
        while (operators.length && !(operators.at(-1)?.type === 'paren' && operators.at(-1)?.value === '(')) {
          output.push(operators.pop() as ExpressionToken)
        }
        operators.pop()
      }
    } else {
      while (
        operators.length
        && operators.at(-1)?.type === 'operator'
        && ((token.value === '^' && precedence[operators.at(-1)!.value] > precedence[token.value])
          || (token.value !== '^' && precedence[operators.at(-1)!.value] >= precedence[token.value]))
      ) output.push(operators.pop() as ExpressionToken)
      operators.push(token)
    }
  }

  while (operators.length) {
    const operator = operators.pop() as ExpressionToken
    if (operator.type === 'paren') throw new Error('unbalanced parentheses')
    output.push(operator)
  }

  const stack: number[] = []
  for (const token of output) {
    if (token.type === 'number') stack.push(token.value)
    else {
      if (token.type !== 'operator' || stack.length < 2) throw new Error('invalid expression')
      const right = stack.pop() as number
      const left = stack.pop() as number
      stack.push(applyOperator(left, right, token.value))
    }
  }
  if (stack.length !== 1) throw new Error('invalid expression')
  return stack[0]
}

function applyOperator(left: number, right: number, operator: string) {
  if (operator === '+') return left + right
  if (operator === '-') return left - right
  if (operator === '*') return left * right
  if (operator === '/') return left / right
  if (operator === '%') return left % right
  if (operator === '^') return left ** right
  throw new Error('unknown operator')
}
