package dom

import (
	"fmt"
	"strings"
)

type htmlTokenKind uint8

const (
	htmlTokenText htmlTokenKind = iota
	htmlTokenStartTag
	htmlTokenEndTag
)

type htmlToken struct {
	kind        htmlTokenKind
	name        string
	text        string
	attrs       []Attribute
	selfClosing bool
}

// This parser is a bounded Phase-1 slice:
// - text/html tokenization + tree construction for explicit tags
// - no legacy/deprecated branches
// - no full optional-tag insertion-mode parity
// It is intentionally conservative while staying aligned with html-standard
// parsing and selector chapters referenced in go/doc/.
var voidElements = map[string]struct{}{
	"area": {}, "base": {}, "br": {}, "col": {}, "embed": {}, "hr": {},
	"img": {}, "input": {}, "link": {}, "meta": {}, "param": {}, "source": {},
	"track": {}, "wbr": {},
}

func (s *Store) BootstrapHTML(html string) error {
	if s == nil {
		return fmt.Errorf("dom store is nil")
	}

	tokens, err := tokenizeHTML(html)
	if err != nil {
		return err
	}

	next := NewStore()
	next.sourceHTML = html

	stack := []NodeID{next.documentID}
	for _, token := range tokens {
		switch token.kind {
		case htmlTokenText:
			if token.text == "" {
				continue
			}
			nodeID := next.newNode(Node{
				Kind: NodeKindText,
				Text: token.text,
			})
			next.appendChild(stack[len(stack)-1], nodeID)
		case htmlTokenStartTag:
			nodeID := next.newNode(Node{
				Kind:    NodeKindElement,
				TagName: token.name,
				Attrs:   token.attrs,
			})
			next.appendChild(stack[len(stack)-1], nodeID)
			if !token.selfClosing && !isVoidElement(token.name) {
				stack = append(stack, nodeID)
			}
		case htmlTokenEndTag:
			if len(stack) <= 1 {
				return fmt.Errorf("unexpected closing tag </%s>", token.name)
			}
			node := next.nodes[stack[len(stack)-1]]
			if node == nil || node.Kind != NodeKindElement || node.TagName != token.name {
				return fmt.Errorf("unexpected closing tag </%s>", token.name)
			}
			stack = stack[:len(stack)-1]
		}
	}

	next.captureDefaultState()
	*s = *next
	return nil
}

func tokenizeHTML(input string) ([]htmlToken, error) {
	tokens := make([]htmlToken, 0, 16)
	for i := 0; i < len(input); {
		if input[i] != '<' {
			next := strings.IndexByte(input[i:], '<')
			if next == -1 {
				next = len(input) - i
			}
			text := input[i : i+next]
			tokens = append(tokens, htmlToken{
				kind: htmlTokenText,
				text: text,
			})
			i += next
			continue
		}

		switch {
		case strings.HasPrefix(input[i:], "<!--"):
			end := strings.Index(input[i+4:], "-->")
			if end == -1 {
				return nil, fmt.Errorf("unterminated HTML comment")
			}
			i += 4 + end + 3
		case strings.HasPrefix(input[i:], "</"):
			token, next, err := parseEndTag(input, i)
			if err != nil {
				return nil, err
			}
			tokens = append(tokens, token)
			i = next
		case strings.HasPrefix(input[i:], "<!"):
			end := strings.IndexByte(input[i:], '>')
			if end == -1 {
				return nil, fmt.Errorf("unterminated markup declaration")
			}
			i += end + 1
		default:
			token, next, err := parseStartTag(input, i)
			if err != nil {
				return nil, err
			}
			tokens = append(tokens, token)
			i = next
			if token.kind == htmlTokenStartTag && token.name == "script" && !token.selfClosing {
				lowerRest := strings.ToLower(input[i:])
				closeOffset := strings.Index(lowerRest, "</script")
				if closeOffset == -1 {
					return nil, fmt.Errorf("unterminated script element")
				}
				if closeOffset > 0 {
					tokens = append(tokens, htmlToken{
						kind: htmlTokenText,
						text: input[i : i+closeOffset],
					})
				}
				endToken, endNext, err := parseEndTag(input, i+closeOffset)
				if err != nil {
					return nil, err
				}
				tokens = append(tokens, endToken)
				i = endNext
			}
		}
	}
	return tokens, nil
}

func (s *Store) captureDefaultState() {
	if s == nil {
		return
	}
	for _, node := range s.nodes {
		if node == nil {
			continue
		}
		node.DefaultAttrs = cloneAttributes(node.Attrs)
		switch node.Kind {
		case NodeKindText:
			node.DefaultText = node.Text
		case NodeKindElement:
			if node.TagName == "textarea" || node.TagName == "option" {
				node.DefaultText = s.TextContentForNode(node.ID)
			}
		}
	}
}

func parseEndTag(input string, start int) (htmlToken, int, error) {
	i := start + 2
	i = skipSpaces(input, i)
	nameStart := i
	for i < len(input) && isNameChar(input[i]) {
		i++
	}
	if nameStart == i {
		return htmlToken{}, 0, fmt.Errorf("invalid closing tag at byte %d", start)
	}
	name := strings.ToLower(input[nameStart:i])
	i = skipSpaces(input, i)
	if i >= len(input) || input[i] != '>' {
		return htmlToken{}, 0, fmt.Errorf("malformed closing tag </%s>", name)
	}
	return htmlToken{
		kind: htmlTokenEndTag,
		name: name,
	}, i + 1, nil
}

func parseStartTag(input string, start int) (htmlToken, int, error) {
	i := start + 1
	i = skipSpaces(input, i)
	nameStart := i
	for i < len(input) && isNameChar(input[i]) {
		i++
	}
	if nameStart == i {
		return htmlToken{}, 0, fmt.Errorf("invalid start tag at byte %d", start)
	}
	token := htmlToken{
		kind: htmlTokenStartTag,
		name: strings.ToLower(input[nameStart:i]),
	}

	for {
		i = skipSpaces(input, i)
		if i >= len(input) {
			return htmlToken{}, 0, fmt.Errorf("unterminated start tag <%s>", token.name)
		}
		if input[i] == '>' {
			return token, i + 1, nil
		}
		if input[i] == '/' {
			if i+1 >= len(input) || input[i+1] != '>' {
				return htmlToken{}, 0, fmt.Errorf("malformed self-closing tag <%s/>", token.name)
			}
			token.selfClosing = true
			return token, i + 2, nil
		}

		attrNameStart := i
		for i < len(input) && isAttrNameChar(input[i]) {
			i++
		}
		if attrNameStart == i {
			return htmlToken{}, 0, fmt.Errorf("invalid attribute in <%s>", token.name)
		}
		attr := Attribute{
			Name: strings.ToLower(input[attrNameStart:i]),
		}

		i = skipSpaces(input, i)
		if i < len(input) && input[i] == '=' {
			attr.HasValue = true
			i++
			i = skipSpaces(input, i)
			if i >= len(input) {
				return htmlToken{}, 0, fmt.Errorf("unterminated attribute `%s` in <%s>", attr.Name, token.name)
			}
			switch input[i] {
			case '"', '\'':
				quote := input[i]
				i++
				valueStart := i
				for i < len(input) && input[i] != quote {
					i++
				}
				if i >= len(input) {
					return htmlToken{}, 0, fmt.Errorf("unterminated quoted attribute `%s` in <%s>", attr.Name, token.name)
				}
				attr.Value = input[valueStart:i]
				i++
			default:
				valueStart := i
				for i < len(input) && !isUnquotedAttrValueTerminator(input, i) {
					i++
				}
				if valueStart == i {
					return htmlToken{}, 0, fmt.Errorf("empty unquoted attribute value `%s` in <%s>", attr.Name, token.name)
				}
				attr.Value = input[valueStart:i]
			}
		}

		token.attrs = append(token.attrs, attr)
	}
}

func isUnquotedAttrValueTerminator(input string, i int) bool {
	if isSpace(input[i]) || input[i] == '>' {
		return true
	}
	return input[i] == '/' && i+1 < len(input) && input[i+1] == '>'
}

func isVoidElement(tagName string) bool {
	_, ok := voidElements[tagName]
	return ok
}

func skipSpaces(text string, i int) int {
	for i < len(text) && isSpace(text[i]) {
		i++
	}
	return i
}

func isSpace(ch byte) bool {
	return ch == ' ' || ch == '\n' || ch == '\r' || ch == '\t' || ch == '\f'
}

func isNameChar(ch byte) bool {
	return (ch >= 'a' && ch <= 'z') ||
		(ch >= 'A' && ch <= 'Z') ||
		(ch >= '0' && ch <= '9') ||
		ch == '-' || ch == '_' || ch == ':'
}

func isAttrNameChar(ch byte) bool {
	if ch == '=' || ch == '/' || ch == '>' || isSpace(ch) {
		return false
	}
	return true
}
