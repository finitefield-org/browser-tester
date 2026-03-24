package script

import (
	"fmt"
	"strconv"
	"strings"
)

func parseHostInvocation(source string) (string, []Value, error) {
	text := strings.TrimSpace(source)
	if text == "" {
		return "", nil, fmt.Errorf("host dispatch requires a non-empty method name")
	}

	open := strings.IndexByte(text, '(')
	if open == -1 {
		if strings.ContainsAny(text, " \t\r\n") {
			return "", nil, fmt.Errorf("host dispatch requires a simple method name")
		}
		return text, nil, nil
	}

	method := strings.TrimSpace(text[:open])
	if method == "" {
		return "", nil, fmt.Errorf("host dispatch requires a non-empty method name")
	}
	if !strings.HasSuffix(text, ")") {
		return "", nil, fmt.Errorf("host dispatch has an unterminated argument list")
	}

	args, err := parseHostArgs(text[open+1 : len(text)-1])
	if err != nil {
		return "", nil, err
	}
	return method, args, nil
}

func parseHostArgs(input string) ([]Value, error) {
	text := strings.TrimSpace(input)
	if text == "" {
		return []Value{}, nil
	}

	args := make([]Value, 0, 4)
	i := 0
	for {
		i = skipHostSpaces(text, i)
		if i >= len(text) {
			break
		}

		value, next, err := parseHostArg(text, i)
		if err != nil {
			return nil, err
		}
		args = append(args, value)

		i = skipHostSpaces(text, next)
		if i >= len(text) {
			break
		}
		if text[i] != ',' {
			return nil, fmt.Errorf("host dispatch arguments must be comma-separated")
		}
		i++
	}
	return args, nil
}

func parseHostArg(input string, start int) (Value, int, error) {
	if start >= len(input) {
		return Value{}, 0, fmt.Errorf("host dispatch argument is missing")
	}

	switch input[start] {
	case '"', '\'':
		quote := input[start]
		i := start + 1
		var b strings.Builder
		for i < len(input) {
			if input[i] == quote {
				return StringValue(b.String()), i + 1, nil
			}
			b.WriteByte(input[i])
			i++
		}
		return Value{}, 0, fmt.Errorf("unterminated quoted host argument")
	default:
		i := start
		for i < len(input) && input[i] != ',' {
			i++
		}
		raw := strings.TrimSpace(input[start:i])
		if raw == "" {
			return Value{}, 0, fmt.Errorf("host dispatch argument is missing")
		}
		if strings.HasPrefix(raw, "expr(") && strings.HasSuffix(raw, ")") {
			inner := strings.TrimSpace(raw[len("expr(") : len(raw)-1])
			if inner == "" {
				return Value{}, 0, fmt.Errorf("expression wrapper requires a non-empty source")
			}
			return InvocationValue(inner), i, nil
		}
		if strings.EqualFold(raw, "true") {
			return BoolValue(true), i, nil
		}
		if strings.EqualFold(raw, "false") {
			return BoolValue(false), i, nil
		}
		if number, err := strconv.ParseFloat(raw, 64); err == nil {
			return NumberValue(number), i, nil
		}
		return StringValue(raw), i, nil
	}
}

func skipHostSpaces(text string, i int) int {
	for i < len(text) {
		switch text[i] {
		case ' ', '\t', '\n', '\r', '\f':
			i++
		default:
			return i
		}
	}
	return i
}
