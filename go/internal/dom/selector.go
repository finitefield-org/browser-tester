package dom

import (
	"fmt"
	"strconv"
	"strings"
	"unicode"
)

type simpleSelector struct {
	tag           string
	anyTag        bool
	id            string
	classes       []string
	attrs         []selectorAttributeCondition
	pseudos       []selectorPseudoClass
	matchGroups   []selectorHasGroup
	hasGroups     []selectorHasGroup
	notGroups     []selectorHasGroup
	nthChild      *selectorNthPattern
	nthOfType     *selectorNthPattern
	nthLastChild  *selectorNthPattern
	nthLastOfType *selectorNthPattern
	stateTags     []string
	langTag       string
	dirTag        string
	headingLevels []int
}

type selectorAttributeCondition struct {
	name            string
	value           string
	operator        selectorAttributeOperator
	caseInsensitive bool
}

type selectorAttributeOperator uint8

const (
	selectorAttributeExists selectorAttributeOperator = iota
	selectorAttributeEquals
	selectorAttributeIncludes
	selectorAttributeDashMatch
	selectorAttributePrefix
	selectorAttributeSuffix
	selectorAttributeSubstring
)

type selectorHasGroup []selectorSequence

type selectorNthPattern struct {
	a int
	b int
}

type selectorPseudoClass uint8

const (
	selectorPseudoRoot selectorPseudoClass = iota
	selectorPseudoScope
	selectorPseudoDefined
	selectorPseudoActive
	selectorPseudoHover
	selectorPseudoEmpty
	selectorPseudoChecked
	selectorPseudoDisabled
	selectorPseudoEnabled
	selectorPseudoFirstChild
	selectorPseudoLastChild
	selectorPseudoLink
	selectorPseudoAnyLink
	selectorPseudoLocalLink
	selectorPseudoVisited
	selectorPseudoDefault
	selectorPseudoPlaceholderShown
	selectorPseudoBlank
	selectorPseudoAutofill
	selectorPseudoRequired
	selectorPseudoOptional
	selectorPseudoReadOnly
	selectorPseudoReadWrite
	selectorPseudoHeading
	selectorPseudoPlaying
	selectorPseudoPaused
	selectorPseudoSeeking
	selectorPseudoBuffering
	selectorPseudoStalled
	selectorPseudoMuted
	selectorPseudoVolumeLocked
	selectorPseudoModal
	selectorPseudoPopoverOpen
	selectorPseudoOpen
	selectorPseudoFocus
	selectorPseudoFocusVisible
	selectorPseudoFocusWithin
	selectorPseudoTarget
	selectorPseudoTargetWithin
	selectorPseudoFirstOfType
	selectorPseudoLastOfType
	selectorPseudoOnlyChild
	selectorPseudoOnlyOfType
	selectorPseudoNthChild
	selectorPseudoNthOfType
	selectorPseudoValid
	selectorPseudoInvalid
	selectorPseudoIndeterminate
	selectorPseudoUserValid
	selectorPseudoUserInvalid
	selectorPseudoInRange
	selectorPseudoOutOfRange
)

type selectorCombinator uint8

const (
	selectorCombinatorNone selectorCombinator = iota
	selectorCombinatorDescendant
	selectorCombinatorChild
	selectorCombinatorAdjacentSibling
	selectorCombinatorGeneralSibling
)

type selectorSequence struct {
	parts []selectorSequencePart
}

type selectorSequencePart struct {
	compound   simpleSelector
	combinator selectorCombinator
}

func (s *Store) Select(selector string) ([]NodeID, error) {
	if s == nil {
		return nil, fmt.Errorf("dom store is nil")
	}
	parsed, err := parseSelectorSequence(selector)
	if err != nil {
		return nil, err
	}

	matches := make([]NodeID, 0, 4)
	for _, rootID := range s.documentChildren() {
		s.walkElementPreOrder(rootID, func(node *Node) {
			if parsed.matchesWithScope(s, node, 0) {
				matches = append(matches, node.ID)
			}
		})
	}
	return matches, nil
}

func (s *Store) walkElementPreOrder(id NodeID, visit func(*Node)) {
	node := s.nodes[id]
	if node == nil {
		return
	}
	if node.Kind == NodeKindElement {
		visit(node)
	}
	for _, childID := range node.Children {
		s.walkElementPreOrder(childID, visit)
	}
}

func parseSelectorSequence(input string) (selectorSequence, error) {
	text := strings.TrimSpace(input)
	if text == "" {
		return selectorSequence{}, fmt.Errorf("selector must not be empty")
	}

	parts := make([]selectorSequencePart, 0, 4)
	i := 0
	for {
		i = skipSpaces(text, i)
		if i >= len(text) {
			break
		}
		if isSelectorCombinator(text[i]) {
			return selectorSequence{}, fmt.Errorf("unsupported selector `%s`: combinators must separate selector compounds", input)
		}

		start := i
		i = scanSelectorCompoundEnd(text, i)
		if start == i {
			return selectorSequence{}, fmt.Errorf("unsupported selector `%s`", input)
		}

		compound, err := parseSimpleSelector(text[start:i])
		if err != nil {
			return selectorSequence{}, err
		}
		parts = append(parts, selectorSequencePart{compound: compound})

		j := i
		hadSpace := false
		for j < len(text) && isSpace(text[j]) {
			hadSpace = true
			j++
		}
		if j >= len(text) {
			break
		}

		switch text[j] {
		case '>':
			parts[len(parts)-1].combinator = selectorCombinatorChild
			i = j + 1
		case '+':
			parts[len(parts)-1].combinator = selectorCombinatorAdjacentSibling
			i = j + 1
		case '~':
			parts[len(parts)-1].combinator = selectorCombinatorGeneralSibling
			i = j + 1
		default:
			if hadSpace {
				parts[len(parts)-1].combinator = selectorCombinatorDescendant
				i = j
			} else {
				return selectorSequence{}, fmt.Errorf("unsupported selector `%s`", input)
			}
		}
	}

	if len(parts) == 0 {
		return selectorSequence{}, fmt.Errorf("selector must not be empty")
	}
	if parts[len(parts)-1].combinator != selectorCombinatorNone {
		return selectorSequence{}, fmt.Errorf("unsupported selector `%s`: trailing combinator", input)
	}
	return selectorSequence{parts: parts}, nil
}

func (s selectorSequence) matches(store *Store, node *Node) bool {
	return s.matchesWithScope(store, node, 0)
}

func (s selectorSequence) matchesWithScope(store *Store, node *Node, scopeNodeID NodeID) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}
	if len(s.parts) == 0 {
		return false
	}

	last := len(s.parts) - 1
	if !s.parts[last].compound.matchesWithScope(store, node, scopeNodeID) {
		return false
	}

	current := node
	for i := last - 1; i >= 0; i-- {
		matched, ok := s.parts[i].matchPredecessor(store, current, scopeNodeID)
		if !ok {
			return false
		}
		current = matched
	}

	return true
}

func (p selectorSequencePart) matchPredecessor(store *Store, current *Node, scopeNodeID NodeID) (*Node, bool) {
	if store == nil || current == nil {
		return nil, false
	}

	switch p.combinator {
	case selectorCombinatorChild:
		parentID := current.Parent
		if parentID == 0 {
			return nil, false
		}
		parent := store.Node(parentID)
		if parent == nil || !p.compound.matchesWithScope(store, parent, scopeNodeID) {
			return nil, false
		}
		return parent, true
	case selectorCombinatorDescendant:
		parentID := current.Parent
		for parentID != 0 {
			parent := store.Node(parentID)
			if parent == nil {
				return nil, false
			}
			if p.compound.matchesWithScope(store, parent, scopeNodeID) {
				return parent, true
			}
			parentID = parent.Parent
		}
		return nil, false
	case selectorCombinatorAdjacentSibling:
		sibling := previousElementSibling(store, current)
		if sibling == nil || !p.compound.matchesWithScope(store, sibling, scopeNodeID) {
			return nil, false
		}
		return sibling, true
	case selectorCombinatorGeneralSibling:
		sibling := previousMatchingSibling(store, current, p.compound, scopeNodeID)
		if sibling == nil {
			return nil, false
		}
		return sibling, true
	default:
		return nil, false
	}
}

func parseSimpleSelector(input string) (simpleSelector, error) {
	text := strings.TrimSpace(input)
	if text == "" {
		return simpleSelector{}, fmt.Errorf("selector must not be empty")
	}

	out := simpleSelector{}
	i := 0
	if text[0] == '*' {
		out.anyTag = true
		i++
	} else if isSelectorNameStart(text[0]) {
		start := i
		i++
		for i < len(text) && isSelectorNameChar(text[i]) {
			i++
		}
		out.tag = strings.ToLower(text[start:i])
	}

	for i < len(text) {
		switch text[i] {
		case '#':
			i++
			start := i
			for i < len(text) && isSelectorNameChar(text[i]) {
				i++
			}
			if start == i {
				return simpleSelector{}, fmt.Errorf("invalid id selector `%s`", input)
			}
			if out.id != "" {
				return simpleSelector{}, fmt.Errorf("multiple id selectors are not supported: `%s`", input)
			}
			out.id = text[start:i]
		case '.':
			i++
			start := i
			for i < len(text) && isSelectorNameChar(text[i]) {
				i++
			}
			if start == i {
				return simpleSelector{}, fmt.Errorf("invalid class selector `%s`", input)
			}
			out.classes = append(out.classes, text[start:i])
		case '[':
			attribute, next, err := parseSelectorAttributeSelector(input, text, i)
			if err != nil {
				return simpleSelector{}, err
			}
			out.attrs = append(out.attrs, attribute)
			i = next
		case ':':
			if strings.HasPrefix(strings.ToLower(text[i:]), ":lang(") {
				if out.langTag != "" {
					return simpleSelector{}, fmt.Errorf("multiple :lang() pseudo-classes are not supported: `%s`", input)
				}
				langTag, next, err := parseSelectorLangPseudoClass(input, text, i)
				if err != nil {
					return simpleSelector{}, err
				}
				out.langTag = langTag
				i = next
				continue
			}
			if strings.HasPrefix(strings.ToLower(text[i:]), ":dir(") {
				if out.dirTag != "" {
					return simpleSelector{}, fmt.Errorf("multiple :dir() pseudo-classes are not supported: `%s`", input)
				}
				dirTag, next, err := parseSelectorDirPseudoClass(input, text, i)
				if err != nil {
					return simpleSelector{}, err
				}
				out.dirTag = dirTag
				i = next
				continue
			}
			if strings.HasPrefix(strings.ToLower(text[i:]), ":heading(") {
				if len(out.headingLevels) > 0 {
					return simpleSelector{}, fmt.Errorf("multiple :heading() pseudo-classes are not supported: `%s`", input)
				}
				headingLevels, next, err := parseSelectorHeadingPseudoClass(input, text, i)
				if err != nil {
					return simpleSelector{}, err
				}
				out.headingLevels = headingLevels
				i = next
				continue
			}
			if strings.HasPrefix(strings.ToLower(text[i:]), ":is(") {
				matchGroup, next, err := parseSelectorIsPseudoClass(input, text, i)
				if err != nil {
					return simpleSelector{}, err
				}
				out.matchGroups = append(out.matchGroups, matchGroup)
				i = next
				continue
			}
			if strings.HasPrefix(strings.ToLower(text[i:]), ":where(") {
				matchGroup, next, err := parseSelectorWherePseudoClass(input, text, i)
				if err != nil {
					return simpleSelector{}, err
				}
				out.matchGroups = append(out.matchGroups, matchGroup)
				i = next
				continue
			}
			if strings.HasPrefix(strings.ToLower(text[i:]), ":nth-child(") {
				if out.nthChild != nil {
					return simpleSelector{}, fmt.Errorf("multiple :nth-child() pseudo-classes are not supported: `%s`", input)
				}
				pattern, next, err := parseSelectorNthPseudoClass(input, text, i, "nth-child")
				if err != nil {
					return simpleSelector{}, err
				}
				out.nthChild = pattern
				i = next
				continue
			}
			if strings.HasPrefix(strings.ToLower(text[i:]), ":nth-of-type(") {
				if out.nthOfType != nil {
					return simpleSelector{}, fmt.Errorf("multiple :nth-of-type() pseudo-classes are not supported: `%s`", input)
				}
				pattern, next, err := parseSelectorNthPseudoClass(input, text, i, "nth-of-type")
				if err != nil {
					return simpleSelector{}, err
				}
				out.nthOfType = pattern
				i = next
				continue
			}
			if strings.HasPrefix(strings.ToLower(text[i:]), ":nth-last-child(") {
				if out.nthLastChild != nil {
					return simpleSelector{}, fmt.Errorf("multiple :nth-last-child() pseudo-classes are not supported: `%s`", input)
				}
				pattern, next, err := parseSelectorNthPseudoClass(input, text, i, "nth-last-child")
				if err != nil {
					return simpleSelector{}, err
				}
				out.nthLastChild = pattern
				i = next
				continue
			}
			if strings.HasPrefix(strings.ToLower(text[i:]), ":nth-last-of-type(") {
				if out.nthLastOfType != nil {
					return simpleSelector{}, fmt.Errorf("multiple :nth-last-of-type() pseudo-classes are not supported: `%s`", input)
				}
				pattern, next, err := parseSelectorNthPseudoClass(input, text, i, "nth-last-of-type")
				if err != nil {
					return simpleSelector{}, err
				}
				out.nthLastOfType = pattern
				i = next
				continue
			}
			if strings.HasPrefix(strings.ToLower(text[i:]), ":has(") {
				hasGroup, next, err := parseSelectorHasPseudoClass(input, text, i)
				if err != nil {
					return simpleSelector{}, err
				}
				out.hasGroups = append(out.hasGroups, hasGroup)
				i = next
				continue
			}
			if strings.HasPrefix(strings.ToLower(text[i:]), ":not(") {
				notGroup, next, err := parseSelectorNotPseudoClass(input, text, i)
				if err != nil {
					return simpleSelector{}, err
				}
				out.notGroups = append(out.notGroups, notGroup)
				i = next
				continue
			}
			if strings.HasPrefix(strings.ToLower(text[i:]), ":state(") {
				stateTag, next, err := parseSelectorStatePseudoClass(input, text, i)
				if err != nil {
					return simpleSelector{}, err
				}
				out.stateTags = append(out.stateTags, stateTag)
				i = next
				continue
			}
			i++
			start := i
			for i < len(text) && isSelectorNameChar(text[i]) {
				i++
			}
			if start == i {
				return simpleSelector{}, fmt.Errorf("invalid pseudo-class selector `%s`", input)
			}
			pseudo, ok := parseSelectorPseudoClass(strings.ToLower(text[start:i]))
			if !ok {
				return simpleSelector{}, fmt.Errorf("unsupported pseudo-class `:%s` in selector `%s`", text[start:i], input)
			}
			out.pseudos = append(out.pseudos, pseudo)
		default:
			return simpleSelector{}, fmt.Errorf("unsupported selector `%s`", input)
		}
	}

	if !out.anyTag && out.tag == "" && out.id == "" && len(out.classes) == 0 && len(out.attrs) == 0 && len(out.pseudos) == 0 && len(out.matchGroups) == 0 && len(out.hasGroups) == 0 && len(out.notGroups) == 0 && out.nthChild == nil && out.nthOfType == nil && out.nthLastChild == nil && out.nthLastOfType == nil && len(out.stateTags) == 0 && len(out.headingLevels) == 0 {
		return simpleSelector{}, fmt.Errorf("selector must include tag, id, class, attribute, pseudo-class, state, heading level, selector-list condition, has condition, not condition, or nth condition")
	}
	return out, nil
}

func (s simpleSelector) matches(store *Store, node *Node) bool {
	return s.matchesWithScope(store, node, 0)
}

func (s simpleSelector) matchesWithScope(store *Store, node *Node, scopeNodeID NodeID) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}
	if !s.anyTag && s.tag != "" && node.TagName != s.tag {
		return false
	}
	if s.id != "" {
		id, ok := attributeValue(node.Attrs, "id")
		if !ok || id != s.id {
			return false
		}
	}
	if len(s.classes) > 0 {
		classValue, ok := attributeValue(node.Attrs, "class")
		if !ok {
			return false
		}
		classList := strings.Fields(classValue)
		for _, expected := range s.classes {
			if !containsToken(classList, expected) {
				return false
			}
		}
	}
	for _, expected := range s.attrs {
		value, ok := attributeValue(node.Attrs, expected.name)
		if !matchesSelectorAttributeCondition(ok, value, expected) {
			return false
		}
	}
	for _, pseudo := range s.pseudos {
		if !pseudo.matchesWithScope(store, node, scopeNodeID) {
			return false
		}
	}
	if s.langTag != "" && !pseudoClassLang(store, node, s.langTag) {
		return false
	}
	if s.dirTag != "" && !pseudoClassDir(store, node, s.dirTag) {
		return false
	}
	for _, expected := range s.stateTags {
		if !pseudoClassState(store, node, expected) {
			return false
		}
	}
	for _, group := range s.hasGroups {
		if !pseudoClassHas(store, node, group, scopeNodeID) {
			return false
		}
	}
	for _, group := range s.matchGroups {
		if !pseudoClassSelectorListMatchesNode(store, node, group, scopeNodeID) {
			return false
		}
	}
	for _, group := range s.notGroups {
		if pseudoClassSelectorListMatchesNode(store, node, group, scopeNodeID) {
			return false
		}
	}
	if s.nthChild != nil && !pseudoClassNthChild(store, node, s.nthChild) {
		return false
	}
	if s.nthOfType != nil && !pseudoClassNthOfType(store, node, s.nthOfType) {
		return false
	}
	if s.nthLastChild != nil && !pseudoClassNthLastChild(store, node, s.nthLastChild) {
		return false
	}
	if s.nthLastOfType != nil && !pseudoClassNthLastOfType(store, node, s.nthLastOfType) {
		return false
	}
	if len(s.headingLevels) > 0 && !pseudoClassHeadingLevel(node, s.headingLevels) {
		return false
	}
	return true
}

func parseSelectorLangPseudoClass(input, text string, index int) (string, int, error) {
	const prefix = ":lang("
	if index < 0 || index+len(prefix) > len(text) {
		return "", 0, fmt.Errorf("invalid :lang() pseudo-class selector `%s`", input)
	}
	if !strings.EqualFold(text[index:index+len(prefix)], prefix) {
		return "", 0, fmt.Errorf("invalid :lang() pseudo-class selector `%s`", input)
	}

	start := index + len(prefix)
	end := strings.IndexByte(text[start:], ')')
	if end < 0 {
		return "", 0, fmt.Errorf("unterminated :lang() pseudo-class selector `%s`", input)
	}
	langTag := strings.ToLower(strings.TrimSpace(text[start : start+end]))
	if !isLanguageTag(langTag) {
		return "", 0, fmt.Errorf("invalid :lang() pseudo-class selector `%s`", input)
	}
	return langTag, start + end + 1, nil
}

func parseSelectorDirPseudoClass(input, text string, index int) (string, int, error) {
	const prefix = ":dir("
	if index < 0 || index+len(prefix) > len(text) {
		return "", 0, fmt.Errorf("invalid :dir() pseudo-class selector `%s`", input)
	}
	if !strings.EqualFold(text[index:index+len(prefix)], prefix) {
		return "", 0, fmt.Errorf("invalid :dir() pseudo-class selector `%s`", input)
	}

	start := index + len(prefix)
	end := strings.IndexByte(text[start:], ')')
	if end < 0 {
		return "", 0, fmt.Errorf("unterminated :dir() pseudo-class selector `%s`", input)
	}
	dirTag := strings.ToLower(strings.TrimSpace(text[start : start+end]))
	switch dirTag {
	case "ltr", "rtl":
		return dirTag, start + end + 1, nil
	default:
		return "", 0, fmt.Errorf("invalid :dir() pseudo-class selector `%s`", input)
	}
}

func parseSelectorHeadingPseudoClass(input, text string, index int) ([]int, int, error) {
	const prefix = ":heading("
	if index < 0 || index+len(prefix) > len(text) {
		return nil, 0, fmt.Errorf("invalid :heading() pseudo-class selector `%s`", input)
	}
	if !strings.EqualFold(text[index:index+len(prefix)], prefix) {
		return nil, 0, fmt.Errorf("invalid :heading() pseudo-class selector `%s`", input)
	}

	start := index + len(prefix)
	end := strings.IndexByte(text[start:], ')')
	if end < 0 {
		return nil, 0, fmt.Errorf("unterminated :heading() pseudo-class selector `%s`", input)
	}
	raw := strings.TrimSpace(text[start : start+end])
	if raw == "" {
		return nil, 0, fmt.Errorf("invalid :heading() pseudo-class selector `%s`", input)
	}

	parts := strings.Split(raw, ",")
	levels := make([]int, 0, len(parts))
	seen := make(map[int]struct{}, len(parts))
	for _, part := range parts {
		levelText := strings.TrimSpace(part)
		if levelText == "" {
			return nil, 0, fmt.Errorf("invalid :heading() pseudo-class selector `%s`", input)
		}
		level, err := strconv.Atoi(levelText)
		if err != nil || level < 1 || level > 6 {
			return nil, 0, fmt.Errorf("invalid :heading() pseudo-class selector `%s`", input)
		}
		if _, ok := seen[level]; ok {
			continue
		}
		seen[level] = struct{}{}
		levels = append(levels, level)
	}
	if len(levels) == 0 {
		return nil, 0, fmt.Errorf("invalid :heading() pseudo-class selector `%s`", input)
	}
	return levels, start + end + 1, nil
}

func parseSelectorStatePseudoClass(input, text string, index int) (string, int, error) {
	const prefix = ":state("
	if index < 0 || index+len(prefix) > len(text) {
		return "", 0, fmt.Errorf("invalid :state() pseudo-class selector `%s`", input)
	}
	if !strings.EqualFold(text[index:index+len(prefix)], prefix) {
		return "", 0, fmt.Errorf("invalid :state() pseudo-class selector `%s`", input)
	}

	start := index + len(prefix)
	end := strings.IndexByte(text[start:], ')')
	if end < 0 {
		return "", 0, fmt.Errorf("unterminated :state() pseudo-class selector `%s`", input)
	}
	stateName := strings.TrimSpace(text[start : start+end])
	if !isCustomStateIdentifier(stateName) {
		return "", 0, fmt.Errorf("invalid :state() pseudo-class selector `%s`", input)
	}
	return stateName, start + end + 1, nil
}

func parseSelectorAttributeSelector(input, text string, index int) (selectorAttributeCondition, int, error) {
	if index < 0 || index >= len(text) || text[index] != '[' {
		return selectorAttributeCondition{}, 0, fmt.Errorf("invalid attribute selector `%s`", input)
	}

	i := index + 1
	i = skipSpaces(text, i)
	nameStart := i
	if i >= len(text) || !isSelectorNameStart(text[i]) {
		return selectorAttributeCondition{}, 0, fmt.Errorf("invalid attribute selector `%s`", input)
	}
	i++
	for i < len(text) && isSelectorNameChar(text[i]) {
		i++
	}
	name := strings.ToLower(text[nameStart:i])
	i = skipSpaces(text, i)

	attr := selectorAttributeCondition{name: name, operator: selectorAttributeExists}
	if i < len(text) {
		switch text[i] {
		case '=':
			attr.operator = selectorAttributeEquals
			i++
		case '~', '|', '^', '$', '*':
			if i+1 >= len(text) || text[i+1] != '=' {
				return selectorAttributeCondition{}, 0, fmt.Errorf("invalid attribute selector `%s`", input)
			}
			switch text[i] {
			case '~':
				attr.operator = selectorAttributeIncludes
			case '|':
				attr.operator = selectorAttributeDashMatch
			case '^':
				attr.operator = selectorAttributePrefix
			case '$':
				attr.operator = selectorAttributeSuffix
			case '*':
				attr.operator = selectorAttributeSubstring
			}
			i += 2
		}
		if attr.operator != selectorAttributeExists {
			i = skipSpaces(text, i)
			if i >= len(text) {
				return selectorAttributeCondition{}, 0, fmt.Errorf("unterminated attribute selector `%s`", input)
			}
			value, next, err := parseSelectorAttributeValue(input, text, i)
			if err != nil {
				return selectorAttributeCondition{}, 0, err
			}
			attr.value = value
			i = next
			i = skipSpaces(text, i)
			if i < len(text) && (text[i] == 'i' || text[i] == 'I' || text[i] == 's' || text[i] == 'S') {
				attr.caseInsensitive = text[i] == 'i' || text[i] == 'I'
				i++
				i = skipSpaces(text, i)
			}
		}
	}

	if i >= len(text) || text[i] != ']' {
		return selectorAttributeCondition{}, 0, fmt.Errorf("unterminated attribute selector `%s`", input)
	}
	return attr, i + 1, nil
}

func parseSelectorAttributeValue(input, text string, index int) (string, int, error) {
	if index < 0 || index >= len(text) {
		return "", 0, fmt.Errorf("invalid attribute selector `%s`", input)
	}

	switch text[index] {
	case '"', '\'':
		quote := text[index]
		i := index + 1
		start := i
		for i < len(text) && text[i] != quote {
			i++
		}
		if i >= len(text) {
			return "", 0, fmt.Errorf("unterminated quoted attribute selector `%s`", input)
		}
		return text[start:i], i + 1, nil
	default:
		start := index
		i := index
		for i < len(text) && !isSpace(text[i]) && text[i] != ']' {
			i++
		}
		if start == i {
			return "", 0, fmt.Errorf("empty attribute selector value `%s`", input)
		}
		return text[start:i], i, nil
	}
}

func parseSelectorHasPseudoClass(input, text string, index int) (selectorHasGroup, int, error) {
	return parseSelectorListPseudoClass(input, text, index, ":has(", "has")
}

func parseSelectorIsPseudoClass(input, text string, index int) (selectorHasGroup, int, error) {
	return parseSelectorListPseudoClass(input, text, index, ":is(", "is")
}

func parseSelectorWherePseudoClass(input, text string, index int) (selectorHasGroup, int, error) {
	return parseSelectorListPseudoClass(input, text, index, ":where(", "where")
}

func parseSelectorNotPseudoClass(input, text string, index int) (selectorHasGroup, int, error) {
	return parseSelectorListPseudoClass(input, text, index, ":not(", "not")
}

func parseSelectorNthPseudoClass(input, text string, index int, pseudoName string) (*selectorNthPattern, int, error) {
	pattern, next, err := parseSelectorFunctionalPseudoClass(input, text, index, pseudoName)
	if err != nil {
		return nil, 0, err
	}
	return pattern, next, nil
}

func parseSelectorListPseudoClass(input, text string, index int, prefix, pseudoName string) (selectorHasGroup, int, error) {
	errorPrefix := fmt.Sprintf("invalid :%s() pseudo-class selector `%%s`", pseudoName)
	if index < 0 || index+len(prefix) > len(text) {
		return nil, 0, fmt.Errorf(errorPrefix, input)
	}
	if !strings.EqualFold(text[index:index+len(prefix)], prefix) {
		return nil, 0, fmt.Errorf(errorPrefix, input)
	}

	start := index + len(prefix)
	end := start
	depth := 1
	for end < len(text) {
		switch text[end] {
		case '(':
			depth++
		case ')':
			depth--
			if depth == 0 {
				raw := strings.TrimSpace(text[start:end])
				if raw == "" {
					return nil, 0, fmt.Errorf(errorPrefix, input)
				}
				selectorTexts, err := splitSelectorList(raw, pseudoName)
				if err != nil {
					return nil, 0, err
				}
				group := make(selectorHasGroup, 0, len(selectorTexts))
				for _, selectorText := range selectorTexts {
					parsed, err := parseSelectorSequence(selectorText)
					if err != nil {
						return nil, 0, err
					}
					group = append(group, parsed)
				}
				return group, end + 1, nil
			}
		}
		end++
	}

	return nil, 0, fmt.Errorf("unterminated :%s() pseudo-class selector `%s`", pseudoName, input)
}

func parseSelectorFunctionalPseudoClass(input, text string, index int, pseudoName string) (*selectorNthPattern, int, error) {
	prefix := ":" + pseudoName + "("
	errorPrefix := fmt.Sprintf("invalid :%s() pseudo-class selector `%%s`", pseudoName)
	if index < 0 || index+len(prefix) > len(text) {
		return nil, 0, fmt.Errorf(errorPrefix, input)
	}
	if !strings.EqualFold(text[index:index+len(prefix)], prefix) {
		return nil, 0, fmt.Errorf(errorPrefix, input)
	}

	start := index + len(prefix)
	end := strings.IndexByte(text[start:], ')')
	if end < 0 {
		return nil, 0, fmt.Errorf("unterminated :%s() pseudo-class selector `%s`", pseudoName, input)
	}
	raw := strings.TrimSpace(text[start : start+end])
	pattern, err := parseNthPattern(raw)
	if err != nil {
		return nil, 0, fmt.Errorf(errorPrefix, input)
	}
	return pattern, start + end + 1, nil
}

func parseNthPattern(raw string) (*selectorNthPattern, error) {
	normalized := strings.ToLower(strings.ReplaceAll(strings.TrimSpace(raw), " ", ""))
	if normalized == "" {
		return nil, fmt.Errorf("empty nth pattern")
	}
	switch normalized {
	case "odd":
		return &selectorNthPattern{a: 2, b: 1}, nil
	case "even":
		return &selectorNthPattern{a: 2, b: 0}, nil
	}

	if strings.Count(normalized, "n") > 1 {
		return nil, fmt.Errorf("invalid nth pattern")
	}
	if !strings.ContainsRune(normalized, 'n') {
		b, err := strconv.Atoi(normalized)
		if err != nil {
			return nil, err
		}
		return &selectorNthPattern{a: 0, b: b}, nil
	}

	idx := strings.IndexByte(normalized, 'n')
	left := normalized[:idx]
	right := normalized[idx+1:]

	a := 1
	switch left {
	case "", "+":
		a = 1
	case "-":
		a = -1
	default:
		parsedA, err := strconv.Atoi(left)
		if err != nil {
			return nil, err
		}
		a = parsedA
	}

	b := 0
	if right != "" {
		parsedB, err := strconv.Atoi(right)
		if err != nil {
			return nil, err
		}
		b = parsedB
	}

	return &selectorNthPattern{a: a, b: b}, nil
}

func parseSelectorPseudoClass(name string) (selectorPseudoClass, bool) {
	switch name {
	case "root":
		return selectorPseudoRoot, true
	case "scope":
		return selectorPseudoScope, true
	case "defined":
		return selectorPseudoDefined, true
	case "active":
		return selectorPseudoActive, true
	case "hover":
		return selectorPseudoHover, true
	case "empty":
		return selectorPseudoEmpty, true
	case "checked":
		return selectorPseudoChecked, true
	case "disabled":
		return selectorPseudoDisabled, true
	case "enabled":
		return selectorPseudoEnabled, true
	case "first-child":
		return selectorPseudoFirstChild, true
	case "last-child":
		return selectorPseudoLastChild, true
	case "link":
		return selectorPseudoLink, true
	case "any-link":
		return selectorPseudoAnyLink, true
	case "local-link":
		return selectorPseudoLocalLink, true
	case "visited":
		return selectorPseudoVisited, true
	case "default":
		return selectorPseudoDefault, true
	case "placeholder-shown":
		return selectorPseudoPlaceholderShown, true
	case "blank":
		return selectorPseudoBlank, true
	case "autofill":
		return selectorPseudoAutofill, true
	case "-webkit-autofill":
		return selectorPseudoAutofill, true
	case "required":
		return selectorPseudoRequired, true
	case "optional":
		return selectorPseudoOptional, true
	case "read-only":
		return selectorPseudoReadOnly, true
	case "read-write":
		return selectorPseudoReadWrite, true
	case "heading":
		return selectorPseudoHeading, true
	case "playing":
		return selectorPseudoPlaying, true
	case "paused":
		return selectorPseudoPaused, true
	case "seeking":
		return selectorPseudoSeeking, true
	case "buffering":
		return selectorPseudoBuffering, true
	case "stalled":
		return selectorPseudoStalled, true
	case "muted":
		return selectorPseudoMuted, true
	case "volume-locked":
		return selectorPseudoVolumeLocked, true
	case "modal":
		return selectorPseudoModal, true
	case "popover-open":
		return selectorPseudoPopoverOpen, true
	case "open":
		return selectorPseudoOpen, true
	case "focus":
		return selectorPseudoFocus, true
	case "focus-visible":
		return selectorPseudoFocusVisible, true
	case "focus-within":
		return selectorPseudoFocusWithin, true
	case "target":
		return selectorPseudoTarget, true
	case "target-within":
		return selectorPseudoTargetWithin, true
	case "first-of-type":
		return selectorPseudoFirstOfType, true
	case "last-of-type":
		return selectorPseudoLastOfType, true
	case "only-child":
		return selectorPseudoOnlyChild, true
	case "only-of-type":
		return selectorPseudoOnlyOfType, true
	case "valid":
		return selectorPseudoValid, true
	case "invalid":
		return selectorPseudoInvalid, true
	case "indeterminate":
		return selectorPseudoIndeterminate, true
	case "user-valid":
		return selectorPseudoUserValid, true
	case "user-invalid":
		return selectorPseudoUserInvalid, true
	case "in-range":
		return selectorPseudoInRange, true
	case "out-of-range":
		return selectorPseudoOutOfRange, true
	default:
		return 0, false
	}
}

func (p selectorPseudoClass) matches(store *Store, node *Node) bool {
	return p.matchesWithScope(store, node, 0)
}

func (p selectorPseudoClass) matchesWithScope(store *Store, node *Node, scopeNodeID NodeID) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}

	switch p {
	case selectorPseudoRoot:
		return node.Parent == store.documentID
	case selectorPseudoScope:
		if scopeNodeID != 0 {
			return node.ID == scopeNodeID
		}
		return node.Parent == store.documentID
	case selectorPseudoDefined:
		return pseudoClassDefined(node)
	case selectorPseudoActive:
		return pseudoClassActive(store, node)
	case selectorPseudoHover:
		return pseudoClassHover(store, node)
	case selectorPseudoEmpty:
		return len(node.Children) == 0
	case selectorPseudoChecked:
		return pseudoClassChecked(store, node)
	case selectorPseudoDisabled:
		return pseudoClassDisabled(node)
	case selectorPseudoEnabled:
		return pseudoClassEnabled(node)
	case selectorPseudoFirstChild:
		return pseudoClassFirstChild(store, node)
	case selectorPseudoLastChild:
		return pseudoClassLastChild(store, node)
	case selectorPseudoLink:
		return pseudoClassLink(store, node)
	case selectorPseudoAnyLink:
		return pseudoClassAnyLink(node)
	case selectorPseudoLocalLink:
		return pseudoClassLocalLink(store, node)
	case selectorPseudoVisited:
		return pseudoClassVisited(store, node)
	case selectorPseudoDefault:
		return pseudoClassDefault(store, node)
	case selectorPseudoPlaceholderShown:
		return pseudoClassPlaceholderShown(store, node)
	case selectorPseudoBlank:
		return pseudoClassBlank(store, node)
	case selectorPseudoAutofill:
		return pseudoClassAutofill(node)
	case selectorPseudoRequired:
		return pseudoClassRequired(node)
	case selectorPseudoOptional:
		return pseudoClassOptional(node)
	case selectorPseudoReadOnly:
		return pseudoClassReadOnly(node)
	case selectorPseudoReadWrite:
		return pseudoClassReadWrite(node)
	case selectorPseudoHeading:
		return pseudoClassHeading(node)
	case selectorPseudoPlaying:
		return pseudoClassPlaying(node)
	case selectorPseudoPaused:
		return pseudoClassPaused(node)
	case selectorPseudoSeeking:
		return pseudoClassSeeking(node)
	case selectorPseudoBuffering:
		return pseudoClassBuffering(node)
	case selectorPseudoStalled:
		return pseudoClassStalled(node)
	case selectorPseudoMuted:
		return pseudoClassMuted(node)
	case selectorPseudoVolumeLocked:
		return pseudoClassVolumeLocked(node)
	case selectorPseudoModal:
		return pseudoClassModal(node)
	case selectorPseudoPopoverOpen:
		return pseudoClassPopoverOpen(node)
	case selectorPseudoOpen:
		return pseudoClassOpen(node)
	case selectorPseudoFocus:
		return pseudoClassFocus(store, node)
	case selectorPseudoFocusVisible:
		return pseudoClassFocusVisible(store, node)
	case selectorPseudoFocusWithin:
		return pseudoClassFocusWithin(store, node)
	case selectorPseudoTarget:
		return pseudoClassTarget(store, node)
	case selectorPseudoTargetWithin:
		return pseudoClassTargetWithin(store, node)
	case selectorPseudoFirstOfType:
		return pseudoClassFirstOfType(store, node)
	case selectorPseudoLastOfType:
		return pseudoClassLastOfType(store, node)
	case selectorPseudoOnlyChild:
		return pseudoClassOnlyChild(store, node)
	case selectorPseudoOnlyOfType:
		return pseudoClassOnlyOfType(store, node)
	case selectorPseudoValid:
		return pseudoClassValid(store, node)
	case selectorPseudoInvalid:
		return pseudoClassInvalid(store, node)
	case selectorPseudoIndeterminate:
		return pseudoClassIndeterminate(store, node)
	case selectorPseudoUserValid:
		return pseudoClassUserValid(store, node)
	case selectorPseudoUserInvalid:
		return pseudoClassUserInvalid(store, node)
	case selectorPseudoInRange:
		return pseudoClassInRange(store, node)
	case selectorPseudoOutOfRange:
		return pseudoClassOutOfRange(store, node)
	default:
		return false
	}
}

func pseudoClassChecked(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}
	switch node.TagName {
	case "input":
		if !isCheckableInputType(inputType(node)) {
			return false
		}
		_, ok := attributeValue(node.Attrs, "checked")
		return ok
	case "option":
		_, ok := attributeValue(node.Attrs, "selected")
		return ok
	default:
		return false
	}
}

func pseudoClassDisabled(node *Node) bool {
	if node == nil || node.Kind != NodeKindElement {
		return false
	}
	if !isEnabledOrDisabledFormControl(node.TagName) {
		return false
	}
	_, ok := attributeValue(node.Attrs, "disabled")
	return ok
}

func pseudoClassEnabled(node *Node) bool {
	if node == nil || node.Kind != NodeKindElement {
		return false
	}
	if !isEnabledOrDisabledFormControl(node.TagName) {
		return false
	}
	_, ok := attributeValue(node.Attrs, "disabled")
	return !ok
}

func pseudoClassFirstChild(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Parent == 0 {
		return false
	}
	parent := store.Node(node.Parent)
	if parent == nil || len(parent.Children) == 0 {
		return false
	}
	return parent.Children[0] == node.ID
}

func pseudoClassLastChild(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Parent == 0 {
		return false
	}
	parent := store.Node(node.Parent)
	if parent == nil || len(parent.Children) == 0 {
		return false
	}
	return parent.Children[len(parent.Children)-1] == node.ID
}

func pseudoClassLink(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}

	switch node.TagName {
	case "a", "area":
		href, ok := attributeValue(node.Attrs, "href")
		if !ok {
			return false
		}
		baseURL := store.CurrentURL()
		if baseURL == "" {
			return pseudoClassAnyLink(node)
		}
		destination := resolveDocumentLinkURL(baseURL, href)
		if destination == "" {
			return pseudoClassAnyLink(node)
		}
		return !store.HasVisitedURL(destination)
	default:
		return false
	}
}

func pseudoClassAnyLink(node *Node) bool {
	if node == nil || node.Kind != NodeKindElement {
		return false
	}
	switch node.TagName {
	case "a", "area":
		_, ok := attributeValue(node.Attrs, "href")
		return ok
	default:
		return false
	}
}

func pseudoClassLocalLink(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}

	switch node.TagName {
	case "a", "area":
		href, ok := attributeValue(node.Attrs, "href")
		if !ok {
			return false
		}
		baseURL := store.CurrentURL()
		if baseURL == "" {
			return false
		}
		destination := resolveDocumentLinkURL(baseURL, href)
		if destination == "" {
			return false
		}
		return sameDocumentURL(baseURL, destination)
	default:
		return false
	}
}

func pseudoClassVisited(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}

	switch node.TagName {
	case "a", "area":
		href, ok := attributeValue(node.Attrs, "href")
		if !ok {
			return false
		}
		baseURL := store.CurrentURL()
		if baseURL == "" {
			return false
		}
		destination := resolveDocumentLinkURL(baseURL, href)
		if destination == "" {
			return false
		}
		return store.HasVisitedURL(destination)
	default:
		return false
	}
}

func pseudoClassDefault(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}

	switch node.TagName {
	case "input":
		switch inputType(node) {
		case "checkbox", "radio":
			_, ok := attributeValue(node.Attrs, "checked")
			return ok
		}
		if !isSubmitControlLike(node) {
			return false
		}
		return isDefaultSubmitControl(store, node)
	case "button":
		if !isSubmitControlLike(node) {
			return false
		}
		return isDefaultSubmitControl(store, node)
	case "option":
		_, ok := attributeValue(node.Attrs, "selected")
		return ok
	default:
		return false
	}
}

func pseudoClassPlaceholderShown(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}
	placeholder, ok := attributeValue(node.Attrs, "placeholder")
	if !ok || strings.TrimSpace(placeholder) == "" {
		return false
	}

	switch node.TagName {
	case "input", "textarea":
		return store.ValueForNode(node.ID) == ""
	default:
		return false
	}
}

func pseudoClassBlank(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}

	switch node.TagName {
	case "input":
		if !isTextInputType(inputType(node)) {
			return false
		}
		return strings.TrimSpace(store.ValueForNode(node.ID)) == ""
	case "textarea":
		return strings.TrimSpace(store.ValueForNode(node.ID)) == ""
	default:
		return false
	}
}

func pseudoClassAutofill(node *Node) bool {
	if node == nil || node.Kind != NodeKindElement || node.TagName != "input" {
		return false
	}
	_, ok := attributeValue(node.Attrs, "autofill")
	return ok
}

func pseudoClassRequired(node *Node) bool {
	if node == nil || node.Kind != NodeKindElement {
		return false
	}
	switch node.TagName {
	case "input":
		if !isRequiredApplicableInputType(inputType(node)) {
			return false
		}
		_, ok := attributeValue(node.Attrs, "required")
		return ok
	case "select", "textarea":
		_, ok := attributeValue(node.Attrs, "required")
		return ok
	default:
		return false
	}
}

func pseudoClassOptional(node *Node) bool {
	if node == nil || node.Kind != NodeKindElement {
		return false
	}
	switch node.TagName {
	case "input":
		if !isRequiredApplicableInputType(inputType(node)) {
			return false
		}
		_, ok := attributeValue(node.Attrs, "required")
		return !ok
	case "select", "textarea":
		_, ok := attributeValue(node.Attrs, "required")
		return !ok
	default:
		return false
	}
}

func pseudoClassReadOnly(node *Node) bool {
	if node == nil || node.Kind != NodeKindElement {
		return false
	}
	return !pseudoClassReadWrite(node)
}

func pseudoClassReadWrite(node *Node) bool {
	if node == nil || node.Kind != NodeKindElement {
		return false
	}

	if value, ok := attributeValue(node.Attrs, "contenteditable"); ok && strings.ToLower(strings.TrimSpace(value)) != "false" {
		return true
	}

	switch node.TagName {
	case "input":
		if !isTextInputType(inputType(node)) {
			return false
		}
		if _, ok := attributeValue(node.Attrs, "readonly"); ok {
			return false
		}
		if _, ok := attributeValue(node.Attrs, "disabled"); ok {
			return false
		}
		return true
	case "textarea":
		if _, ok := attributeValue(node.Attrs, "readonly"); ok {
			return false
		}
		if _, ok := attributeValue(node.Attrs, "disabled"); ok {
			return false
		}
		return true
	default:
		return false
	}
}

func pseudoClassHeading(node *Node) bool {
	if node == nil || node.Kind != NodeKindElement {
		return false
	}
	_, ok := headingLevel(node)
	return ok
}

func pseudoClassHeadingLevel(node *Node, levels []int) bool {
	if node == nil || node.Kind != NodeKindElement || len(levels) == 0 {
		return false
	}
	level, ok := headingLevel(node)
	if !ok {
		return false
	}
	for _, expected := range levels {
		if expected == level {
			return true
		}
	}
	return false
}

func pseudoClassPlaying(node *Node) bool {
	if !isMediaElement(node) {
		return false
	}
	_, paused := attributeValue(node.Attrs, "paused")
	return !paused
}

func pseudoClassPaused(node *Node) bool {
	if !isMediaElement(node) {
		return false
	}
	_, paused := attributeValue(node.Attrs, "paused")
	return paused
}

func pseudoClassSeeking(node *Node) bool {
	if !isMediaElement(node) {
		return false
	}
	_, seeking := attributeValue(node.Attrs, "seeking")
	return seeking
}

func pseudoClassBuffering(node *Node) bool {
	if !isMediaElement(node) {
		return false
	}
	if _, paused := attributeValue(node.Attrs, "paused"); paused {
		return false
	}
	networkState, ok := attributeValue(node.Attrs, "networkstate")
	if !ok || !strings.EqualFold(strings.TrimSpace(networkState), "loading") {
		return false
	}
	readyState, ok := mediaReadyState(node)
	if !ok {
		return false
	}
	return readyState <= 2
}

func pseudoClassStalled(node *Node) bool {
	if !pseudoClassBuffering(node) {
		return false
	}
	_, stalled := attributeValue(node.Attrs, "stalled")
	return stalled
}

func pseudoClassMuted(node *Node) bool {
	if !isMediaElement(node) {
		return false
	}
	_, muted := attributeValue(node.Attrs, "muted")
	return muted
}

func pseudoClassVolumeLocked(node *Node) bool {
	if !isMediaElement(node) {
		return false
	}
	_, locked := attributeValue(node.Attrs, "volume-locked")
	return locked
}

func pseudoClassDefined(node *Node) bool {
	if node == nil || node.Kind != NodeKindElement {
		return false
	}
	if !strings.Contains(node.TagName, "-") {
		return true
	}
	_, ok := attributeValue(node.Attrs, "defined")
	return ok
}

func pseudoClassState(store *Store, node *Node, expected string) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}
	if expected == "" || !strings.Contains(node.TagName, "-") {
		return false
	}
	value, ok := attributeValue(node.Attrs, "state")
	if !ok {
		return false
	}
	return containsToken(strings.Fields(value), expected)
}

func pseudoClassActive(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}
	if _, ok := attributeValue(node.Attrs, "active"); ok {
		return true
	}
	return subtreeContainsAttribute(store, node.ID, "active")
}

func pseudoClassHover(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}
	if _, ok := attributeValue(node.Attrs, "hover"); ok {
		return true
	}
	return subtreeContainsAttribute(store, node.ID, "hover")
}

func pseudoClassModal(node *Node) bool {
	if node == nil || node.Kind != NodeKindElement {
		return false
	}
	switch node.TagName {
	case "dialog":
		_, ok := attributeValue(node.Attrs, "modal")
		return ok
	default:
		_, ok := attributeValue(node.Attrs, "fullscreen")
		return ok
	}
}

func pseudoClassPopoverOpen(node *Node) bool {
	if node == nil || node.Kind != NodeKindElement {
		return false
	}
	if _, ok := attributeValue(node.Attrs, "popover"); !ok {
		return false
	}
	_, ok := attributeValue(node.Attrs, "popover-open")
	return ok
}

func headingLevel(node *Node) (int, bool) {
	if node == nil || node.Kind != NodeKindElement {
		return 0, false
	}
	switch node.TagName {
	case "h1":
		return 1, true
	case "h2":
		return 2, true
	case "h3":
		return 3, true
	case "h4":
		return 4, true
	case "h5":
		return 5, true
	case "h6":
		return 6, true
	default:
		return 0, false
	}
}

func isMediaElement(node *Node) bool {
	if node == nil || node.Kind != NodeKindElement {
		return false
	}
	switch node.TagName {
	case "audio", "video":
		return true
	default:
		return false
	}
}

func mediaReadyState(node *Node) (int, bool) {
	if !isMediaElement(node) {
		return 0, false
	}
	value, ok := attributeValue(node.Attrs, "readystate")
	if !ok {
		return 0, false
	}
	switch strings.ToLower(strings.TrimSpace(value)) {
	case "0", "have_nothing", "have-nothing":
		return 0, true
	case "1", "have_metadata", "have-metadata":
		return 1, true
	case "2", "have_current_data", "have-current-data":
		return 2, true
	case "3", "have_future_data", "have-future-data":
		return 3, true
	case "4", "have_enough_data", "have-enough-data":
		return 4, true
	default:
		return 0, false
	}
}

func pseudoClassOpen(node *Node) bool {
	if node == nil || node.Kind != NodeKindElement {
		return false
	}
	switch node.TagName {
	case "details", "dialog":
		_, ok := attributeValue(node.Attrs, "open")
		return ok
	default:
		return false
	}
}

func pseudoClassFocus(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}
	return store.focusedNodeID == node.ID
}

func pseudoClassFocusVisible(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}
	if store.focusedNodeID == node.ID {
		return true
	}
	_, ok := attributeValue(node.Attrs, "focus-visible")
	return ok
}

func pseudoClassFocusWithin(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}
	focusedNodeID := store.focusedNodeID
	if focusedNodeID == 0 {
		return false
	}
	if focusedNodeID == node.ID {
		return true
	}
	return subtreeContainsNode(store, node.ID, focusedNodeID)
}

func pseudoClassTarget(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}
	return store.targetNodeID == node.ID
}

func pseudoClassTargetWithin(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}
	targetNodeID := store.targetNodeID
	if targetNodeID == 0 {
		return false
	}
	if targetNodeID == node.ID {
		return true
	}
	return subtreeContainsNode(store, node.ID, targetNodeID)
}

func pseudoClassHas(store *Store, node *Node, group selectorHasGroup, scopeNodeID NodeID) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement || len(group) == 0 {
		return false
	}

	found := false
	store.walkElementPreOrder(node.ID, func(current *Node) {
		if found || current == nil || current.ID == node.ID {
			return
		}
		for _, selector := range group {
			if selector.matchesWithScope(store, current, node.ID) {
				found = true
				return
			}
		}
	})
	return found
}

func pseudoClassSelectorListMatchesNode(store *Store, node *Node, group selectorHasGroup, scopeNodeID NodeID) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement || len(group) == 0 {
		return false
	}
	for _, selector := range group {
		if selector.matchesWithScope(store, node, scopeNodeID) {
			return true
		}
	}
	return false
}

func pseudoClassNthChild(store *Store, node *Node, pattern *selectorNthPattern) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement || node.Parent == 0 || pattern == nil {
		return false
	}
	index, ok := elementChildIndex(store, node)
	if !ok {
		return false
	}
	return pattern.matches(index)
}

func pseudoClassNthOfType(store *Store, node *Node, pattern *selectorNthPattern) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement || node.Parent == 0 || pattern == nil {
		return false
	}
	index, ok := elementOfTypeIndex(store, node)
	if !ok {
		return false
	}
	return pattern.matches(index)
}

func pseudoClassNthLastChild(store *Store, node *Node, pattern *selectorNthPattern) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement || node.Parent == 0 || pattern == nil {
		return false
	}
	index, ok := elementLastChildIndex(store, node)
	if !ok {
		return false
	}
	return pattern.matches(index)
}

func pseudoClassNthLastOfType(store *Store, node *Node, pattern *selectorNthPattern) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement || node.Parent == 0 || pattern == nil {
		return false
	}
	index, ok := elementLastOfTypeIndex(store, node)
	if !ok {
		return false
	}
	return pattern.matches(index)
}

func (p selectorNthPattern) matches(index int) bool {
	if index < 1 {
		return false
	}
	if p.a == 0 {
		return index == p.b
	}
	if p.a > 0 {
		if index < p.b {
			return false
		}
		diff := index - p.b
		return diff%p.a == 0
	}
	if index > p.b {
		return false
	}
	diff := p.b - index
	step := -p.a
	return diff%step == 0
}

func pseudoClassLang(store *Store, node *Node, expected string) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}
	if expected == "" {
		return false
	}
	language := elementLanguage(store, node)
	if language == "" {
		return false
	}
	if language == expected {
		return true
	}
	if len(language) <= len(expected) {
		return false
	}
	return strings.HasPrefix(language, expected) && language[len(expected)] == '-'
}

func pseudoClassDir(store *Store, node *Node, expected string) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}
	if expected != "ltr" && expected != "rtl" {
		return false
	}
	return elementDirection(store, node) == expected
}

func pseudoClassFirstOfType(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement || node.Parent == 0 {
		return false
	}
	parent := store.Node(node.Parent)
	if parent == nil {
		return false
	}
	for _, childID := range parent.Children {
		sibling := store.Node(childID)
		if sibling == nil || sibling.Kind != NodeKindElement {
			continue
		}
		if sibling.ID == node.ID {
			return true
		}
		if sibling.TagName == node.TagName {
			return false
		}
	}
	return false
}

func pseudoClassLastOfType(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement || node.Parent == 0 {
		return false
	}
	parent := store.Node(node.Parent)
	if parent == nil {
		return false
	}
	children := parent.Children
	for i := len(children) - 1; i >= 0; i-- {
		sibling := store.Node(children[i])
		if sibling == nil || sibling.Kind != NodeKindElement {
			continue
		}
		if sibling.ID == node.ID {
			return true
		}
		if sibling.TagName == node.TagName {
			return false
		}
	}
	return false
}

func elementChildIndex(store *Store, node *Node) (int, bool) {
	if store == nil || node == nil || node.Parent == 0 {
		return 0, false
	}
	parent := store.Node(node.Parent)
	if parent == nil {
		return 0, false
	}
	index := 0
	for _, childID := range parent.Children {
		child := store.Node(childID)
		if child == nil || child.Kind != NodeKindElement {
			continue
		}
		index++
		if child.ID == node.ID {
			return index, true
		}
	}
	return 0, false
}

func elementOfTypeIndex(store *Store, node *Node) (int, bool) {
	if store == nil || node == nil || node.Parent == 0 {
		return 0, false
	}
	parent := store.Node(node.Parent)
	if parent == nil {
		return 0, false
	}
	index := 0
	for _, childID := range parent.Children {
		child := store.Node(childID)
		if child == nil || child.Kind != NodeKindElement || child.TagName != node.TagName {
			continue
		}
		index++
		if child.ID == node.ID {
			return index, true
		}
	}
	return 0, false
}

func elementLastChildIndex(store *Store, node *Node) (int, bool) {
	if store == nil || node == nil || node.Parent == 0 {
		return 0, false
	}
	parent := store.Node(node.Parent)
	if parent == nil {
		return 0, false
	}
	index := 0
	for i := len(parent.Children) - 1; i >= 0; i-- {
		child := store.Node(parent.Children[i])
		if child == nil || child.Kind != NodeKindElement {
			continue
		}
		index++
		if child.ID == node.ID {
			return index, true
		}
	}
	return 0, false
}

func elementLastOfTypeIndex(store *Store, node *Node) (int, bool) {
	if store == nil || node == nil || node.Parent == 0 {
		return 0, false
	}
	parent := store.Node(node.Parent)
	if parent == nil {
		return 0, false
	}
	index := 0
	for i := len(parent.Children) - 1; i >= 0; i-- {
		child := store.Node(parent.Children[i])
		if child == nil || child.Kind != NodeKindElement || child.TagName != node.TagName {
			continue
		}
		index++
		if child.ID == node.ID {
			return index, true
		}
	}
	return 0, false
}

func pseudoClassOnlyChild(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}
	return pseudoClassFirstChild(store, node) && pseudoClassLastChild(store, node)
}

func pseudoClassOnlyOfType(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}
	return pseudoClassFirstOfType(store, node) && pseudoClassLastOfType(store, node)
}

func elementLanguage(store *Store, node *Node) string {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return ""
	}

	current := node
	for current != nil {
		if value, ok := attributeValue(current.Attrs, "lang"); ok {
			return strings.ToLower(strings.TrimSpace(value))
		}
		if current.Parent == 0 {
			break
		}
		current = store.Node(current.Parent)
		if current == nil || current.Kind != NodeKindElement {
			break
		}
	}
	return ""
}

func elementDirection(store *Store, node *Node) string {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return "ltr"
	}

	if value, ok := attributeValue(node.Attrs, "dir"); ok {
		switch strings.ToLower(strings.TrimSpace(value)) {
		case "ltr", "rtl":
			return strings.ToLower(strings.TrimSpace(value))
		case "auto":
			if auto := elementAutoDirection(store, node); auto != "" {
				return auto
			}
			return "ltr"
		default:
			if parent := parentDirection(store, node); parent != "" {
				return parent
			}
			return "ltr"
		}
	}

	if parent := parentDirection(store, node); parent != "" {
		return parent
	}
	return "ltr"
}

func parentDirection(store *Store, node *Node) string {
	if store == nil || node == nil || node.Parent == 0 {
		return ""
	}
	parent := store.Node(node.Parent)
	if parent == nil || parent.Kind != NodeKindElement {
		return ""
	}
	return elementDirection(store, parent)
}

func elementAutoDirection(store *Store, node *Node) string {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return ""
	}

	if isAutoDirectionalityInput(node) {
		if direction := firstStrongDirection(store.ValueForNode(node.ID)); direction != "" {
			return direction
		}
		return ""
	}

	if direction := firstStrongDirection(nodeTextContent(store, node)); direction != "" {
		return direction
	}
	return ""
}

func isAutoDirectionalityInput(node *Node) bool {
	if node == nil || node.Kind != NodeKindElement {
		return false
	}
	switch node.TagName {
	case "textarea":
		return true
	case "input":
		switch inputType(node) {
		case "", "text", "search", "url", "tel", "email", "password":
			return true
		default:
			return false
		}
	default:
		return false
	}
}

func nodeTextContent(store *Store, node *Node) string {
	if store == nil || node == nil {
		return ""
	}
	var b strings.Builder
	collectTextContent(store, node, &b)
	return b.String()
}

func collectTextContent(store *Store, node *Node, b *strings.Builder) {
	if store == nil || node == nil || b == nil {
		return
	}
	if node.Kind == NodeKindText {
		b.WriteString(node.Text)
		return
	}
	for _, childID := range node.Children {
		collectTextContent(store, store.Node(childID), b)
	}
}

func firstStrongDirection(text string) string {
	for _, r := range text {
		if isStrongRTL(r) {
			return "rtl"
		}
		if isStrongLTR(r) {
			return "ltr"
		}
	}
	return ""
}

func isStrongLTR(r rune) bool {
	return unicode.IsLetter(r) && !isStrongRTL(r)
}

func isStrongRTL(r rune) bool {
	return unicode.Is(unicode.Hebrew, r) || unicode.Is(unicode.Arabic, r)
}

func isLanguageTag(value string) bool {
	if value == "" {
		return false
	}
	segments := strings.Split(value, "-")
	for _, segment := range segments {
		if segment == "" {
			return false
		}
		for i := 0; i < len(segment); i++ {
			ch := segment[i]
			if !isLanguageTagChar(ch) {
				return false
			}
		}
	}
	return true
}

func isLanguageTagChar(ch byte) bool {
	return (ch >= 'a' && ch <= 'z') || (ch >= '0' && ch <= '9')
}

func pseudoClassValid(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}
	switch node.TagName {
	case "form", "fieldset":
		return !hasInvalidConstraintDescendant(store, node)
	default:
		if !isBoundedConstraintValidationControl(node) {
			return false
		}
		return controlSatisfiesBoundedConstraints(store, node)
	}
}

func pseudoClassInvalid(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}
	switch node.TagName {
	case "form", "fieldset":
		return hasInvalidConstraintDescendant(store, node)
	default:
		if !isBoundedConstraintValidationControl(node) {
			return false
		}
		return !controlSatisfiesBoundedConstraints(store, node)
	}
}

func pseudoClassIndeterminate(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}
	switch node.TagName {
	case "input":
		switch inputType(node) {
		case "checkbox":
			_, ok := attributeValue(node.Attrs, "indeterminate")
			return ok
		case "radio":
			return !radioGroupHasChecked(store, node)
		default:
			return false
		}
	case "progress":
		_, ok := attributeValue(node.Attrs, "value")
		return !ok
	default:
		return false
	}
}

func pseudoClassUserValid(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}
	if !isUserValidityPseudoClassCandidate(node) || !node.UserValidity {
		return false
	}
	return controlSatisfiesBoundedConstraints(store, node)
}

func pseudoClassUserInvalid(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}
	if !isUserValidityPseudoClassCandidate(node) || !node.UserValidity {
		return false
	}
	return !controlSatisfiesBoundedConstraints(store, node)
}

func pseudoClassInRange(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}
	if !isBoundedRangeInput(node) {
		return false
	}
	value := strings.TrimSpace(store.ValueForNode(node.ID))
	if value == "" {
		return true
	}
	numericValue, ok := parseBoundedFloat(value)
	if !ok {
		return false
	}
	minValue, hasMin := boundedFloatAttribute(node, "min")
	maxValue, hasMax := boundedFloatAttribute(node, "max")
	if hasMin && numericValue < minValue {
		return false
	}
	if hasMax && numericValue > maxValue {
		return false
	}
	return true
}

func pseudoClassOutOfRange(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}
	if !isBoundedRangeInput(node) {
		return false
	}
	value := strings.TrimSpace(store.ValueForNode(node.ID))
	if value == "" {
		return false
	}
	numericValue, ok := parseBoundedFloat(value)
	if !ok {
		return false
	}
	minValue, hasMin := boundedFloatAttribute(node, "min")
	maxValue, hasMax := boundedFloatAttribute(node, "max")
	if hasMin && numericValue < minValue {
		return true
	}
	if hasMax && numericValue > maxValue {
		return true
	}
	return false
}

func hasInvalidConstraintDescendant(store *Store, node *Node) bool {
	if store == nil || node == nil {
		return false
	}

	invalid := false
	store.walkElementPreOrder(node.ID, func(current *Node) {
		if invalid || current == nil || current.ID == node.ID {
			return
		}
		if node.TagName == "form" && nearestAncestorForm(store, current.ID) != node.ID {
			return
		}
		if isBoundedConstraintValidationControl(current) && !controlSatisfiesBoundedConstraints(store, current) {
			invalid = true
		}
	})
	return invalid
}

func isBoundedConstraintValidationControl(node *Node) bool {
	if node == nil || node.Kind != NodeKindElement {
		return false
	}
	if _, ok := attributeValue(node.Attrs, "disabled"); ok {
		return false
	}

	switch node.TagName {
	case "input":
		typeName := inputType(node)
		switch typeName {
		case "hidden", "button", "reset", "submit", "image":
			return false
		}
		if _, ok := attributeValue(node.Attrs, "readonly"); ok && isTextInputType(typeName) {
			return false
		}
		return true
	case "select":
		return true
	case "textarea":
		if _, ok := attributeValue(node.Attrs, "readonly"); ok {
			return false
		}
		return true
	case "fieldset":
		return true
	default:
		return false
	}
}

func controlSatisfiesBoundedConstraints(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}
	if !isBoundedConstraintValidationControl(node) {
		return false
	}

	switch node.TagName {
	case "input":
		typeName := inputType(node)
		emptyValue := strings.TrimSpace(store.ValueForNode(node.ID)) == ""
		required := hasAttribute(node.Attrs, "required")
		switch typeName {
		case "checkbox":
			checked, ok := store.CheckedForNode(node.ID)
			if !ok {
				return false
			}
			if required && !checked {
				return false
			}
			return true
		case "radio":
			if radioGroupRequiresChecked(store, node) && !radioGroupHasChecked(store, node) {
				return false
			}
			return true
		case "number", "range":
			if emptyValue {
				return !required
			}
			return boundedNumericValueWithinRange(store, node)
		default:
			return !required || !emptyValue
		}
	case "select":
		if hasAttribute(node.Attrs, "required") && strings.TrimSpace(store.ValueForNode(node.ID)) == "" {
			return false
		}
		return true
	case "textarea":
		if hasAttribute(node.Attrs, "required") && strings.TrimSpace(store.ValueForNode(node.ID)) == "" {
			return false
		}
		return true
	case "fieldset":
		return !hasInvalidConstraintDescendant(store, node)
	default:
		return false
	}
}

func isUserValidityPseudoClassCandidate(node *Node) bool {
	if node == nil || node.Kind != NodeKindElement {
		return false
	}
	if !isBoundedConstraintValidationControl(node) {
		return false
	}
	switch node.TagName {
	case "input", "select", "textarea":
		return true
	default:
		return false
	}
}

func radioGroupRequiresChecked(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement || node.TagName != "input" || inputType(node) != "radio" {
		return false
	}

	currentName, hasName := attributeValue(node.Attrs, "name")
	if !hasName || currentName == "" {
		return hasAttribute(node.Attrs, "required")
	}
	currentForm := nearestAncestorForm(store, node.ID)
	for _, other := range store.Nodes() {
		if other == nil || other.Kind != NodeKindElement || other.TagName != "input" || inputType(other) != "radio" {
			continue
		}
		otherName, otherHasName := attributeValue(other.Attrs, "name")
		if otherHasName != hasName || otherName != currentName {
			continue
		}
		if nearestAncestorForm(store, other.ID) != currentForm {
			continue
		}
		if !isBoundedConstraintValidationControl(other) {
			continue
		}
		if hasAttribute(other.Attrs, "required") {
			return true
		}
	}
	return hasAttribute(node.Attrs, "required")
}

func radioGroupHasChecked(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement || node.TagName != "input" || inputType(node) != "radio" {
		return false
	}

	currentName, hasName := attributeValue(node.Attrs, "name")
	if !hasName || currentName == "" {
		checked, ok := store.CheckedForNode(node.ID)
		if !ok {
			return false
		}
		return checked
	}
	currentForm := nearestAncestorForm(store, node.ID)
	for _, other := range store.Nodes() {
		if other == nil || other.Kind != NodeKindElement || other.TagName != "input" || inputType(other) != "radio" {
			continue
		}
		otherName, otherHasName := attributeValue(other.Attrs, "name")
		if otherHasName != hasName || otherName != currentName {
			continue
		}
		if nearestAncestorForm(store, other.ID) != currentForm {
			continue
		}
		if !isBoundedConstraintValidationControl(other) {
			continue
		}
		if _, ok := attributeValue(other.Attrs, "checked"); ok {
			return true
		}
	}
	return false
}

func boundedNumericValueWithinRange(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}

	value := strings.TrimSpace(store.ValueForNode(node.ID))
	if value == "" {
		return true
	}
	numericValue, ok := parseBoundedFloat(value)
	if !ok {
		return false
	}
	minValue, hasMin := boundedFloatAttribute(node, "min")
	maxValue, hasMax := boundedFloatAttribute(node, "max")
	if hasMin && numericValue < minValue {
		return false
	}
	if hasMax && numericValue > maxValue {
		return false
	}
	return true
}

func isBoundedRangeInput(node *Node) bool {
	if node == nil || node.Kind != NodeKindElement || node.TagName != "input" {
		return false
	}
	switch inputType(node) {
	case "number", "range":
		return true
	default:
		return false
	}
}

func parseBoundedFloat(value string) (float64, bool) {
	value = strings.TrimSpace(value)
	if value == "" {
		return 0, false
	}
	parsed, err := strconv.ParseFloat(value, 64)
	if err != nil {
		return 0, false
	}
	return parsed, true
}

func boundedFloatAttribute(node *Node, name string) (float64, bool) {
	if node == nil {
		return 0, false
	}
	value, ok := attributeValue(node.Attrs, name)
	if !ok {
		return 0, false
	}
	return parseBoundedFloat(value)
}

func isSubmitControlLike(node *Node) bool {
	if node == nil || node.Kind != NodeKindElement {
		return false
	}

	switch node.TagName {
	case "button":
		typeName, _ := attributeValue(node.Attrs, "type")
		typeName = strings.ToLower(strings.TrimSpace(typeName))
		return typeName == "" || typeName == "submit"
	case "input":
		switch inputType(node) {
		case "submit", "image":
			return true
		}
	}

	return false
}

func isDefaultSubmitControl(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement || !isSubmitControlLike(node) {
		return false
	}

	formID := nearestAncestorForm(store, node.ID)
	if formID == 0 {
		return false
	}

	var firstSubmit NodeID
	store.walkElementPreOrder(formID, func(current *Node) {
		if firstSubmit != 0 || current == nil || current.ID == formID {
			return
		}
		if isSubmitControlLike(current) && nearestAncestorForm(store, current.ID) == formID {
			firstSubmit = current.ID
		}
	})
	return firstSubmit == node.ID
}

func isEnabledOrDisabledFormControl(tagName string) bool {
	switch tagName {
	case "button", "input", "select", "textarea", "option", "optgroup", "fieldset":
		return true
	default:
		return false
	}
}

func isRequiredApplicableInputType(typeName string) bool {
	switch strings.ToLower(strings.TrimSpace(typeName)) {
	case "", "text", "search", "url", "tel", "email", "password", "date", "month", "week", "time", "datetime-local", "number", "checkbox", "radio", "file":
		return true
	default:
		return false
	}
}

func isSelectorCompoundTerminator(ch byte) bool {
	return isSpace(ch) || isSelectorCombinator(ch)
}

func isSelectorCombinator(ch byte) bool {
	switch ch {
	case '>', '+', '~':
		return true
	default:
		return false
	}
}

func scanSelectorCompoundEnd(text string, start int) int {
	parenDepth := 0
	bracketDepth := 0
	var quote byte
	for i := start; i < len(text); i++ {
		ch := text[i]
		if quote != 0 {
			if ch == quote {
				quote = 0
			}
			continue
		}
		switch ch {
		case '"', '\'':
			if parenDepth > 0 || bracketDepth > 0 {
				quote = ch
			}
		case '(':
			if bracketDepth == 0 {
				parenDepth++
			}
		case ')':
			if bracketDepth == 0 && parenDepth > 0 {
				parenDepth--
			}
		case '[':
			if parenDepth == 0 {
				bracketDepth++
			}
		case ']':
			if bracketDepth > 0 {
				bracketDepth--
			}
		default:
			if parenDepth == 0 && bracketDepth == 0 && isSelectorCompoundTerminator(ch) {
				return i
			}
		}
	}
	return len(text)
}

func attributeValue(attrs []Attribute, name string) (string, bool) {
	for _, attr := range attrs {
		if attr.Name == name {
			return attr.Value, true
		}
	}
	return "", false
}

func hasAttribute(attrs []Attribute, name string) bool {
	_, ok := attributeValue(attrs, name)
	return ok
}

func matchesSelectorAttributeCondition(has bool, value string, expected selectorAttributeCondition) bool {
	switch expected.operator {
	case selectorAttributeExists:
		return has
	case selectorAttributeEquals:
		if !has {
			return false
		}
		return selectorAttributeValueEquals(value, expected.value, expected.caseInsensitive)
	case selectorAttributeIncludes:
		if !has {
			return false
		}
		return selectorAttributeValueIncludes(value, expected.value, expected.caseInsensitive)
	case selectorAttributeDashMatch:
		if !has {
			return false
		}
		return selectorAttributeValueDashMatch(value, expected.value, expected.caseInsensitive)
	case selectorAttributePrefix:
		if !has {
			return false
		}
		return selectorAttributeValuePrefix(value, expected.value, expected.caseInsensitive)
	case selectorAttributeSuffix:
		if !has {
			return false
		}
		return selectorAttributeValueSuffix(value, expected.value, expected.caseInsensitive)
	case selectorAttributeSubstring:
		if !has {
			return false
		}
		return selectorAttributeValueSubstring(value, expected.value, expected.caseInsensitive)
	default:
		return false
	}
}

func selectorAttributeValueEquals(value, expected string, caseInsensitive bool) bool {
	if caseInsensitive {
		return equalASCIIInsensitive(value, expected)
	}
	return value == expected
}

func selectorAttributeValueIncludes(value, expected string, caseInsensitive bool) bool {
	for _, token := range strings.Fields(value) {
		if selectorAttributeValueEquals(token, expected, caseInsensitive) {
			return true
		}
	}
	return false
}

func selectorAttributeValueDashMatch(value, expected string, caseInsensitive bool) bool {
	if selectorAttributeValueEquals(value, expected, caseInsensitive) {
		return true
	}
	if len(value) < len(expected)+1 {
		return false
	}
	prefix := value[:len(expected)]
	if !selectorAttributeValueEquals(prefix, expected, caseInsensitive) {
		return false
	}
	return value[len(expected)] == '-'
}

func selectorAttributeValuePrefix(value, expected string, caseInsensitive bool) bool {
	if len(value) < len(expected) {
		return false
	}
	return selectorAttributeValueEquals(value[:len(expected)], expected, caseInsensitive)
}

func selectorAttributeValueSuffix(value, expected string, caseInsensitive bool) bool {
	if len(value) < len(expected) {
		return false
	}
	return selectorAttributeValueEquals(value[len(value)-len(expected):], expected, caseInsensitive)
}

func selectorAttributeValueSubstring(value, expected string, caseInsensitive bool) bool {
	if !caseInsensitive {
		return strings.Contains(value, expected)
	}
	if expected == "" {
		return true
	}
	needle := len(expected)
	for start := 0; start+needle <= len(value); start++ {
		if equalASCIIInsensitive(value[start:start+needle], expected) {
			return true
		}
	}
	return false
}

func equalASCIIInsensitive(a, b string) bool {
	if len(a) != len(b) {
		return false
	}
	for i := 0; i < len(a); i++ {
		ca := a[i]
		cb := b[i]
		if 'A' <= ca && ca <= 'Z' {
			ca += 'a' - 'A'
		}
		if 'A' <= cb && cb <= 'Z' {
			cb += 'a' - 'A'
		}
		if ca != cb {
			return false
		}
	}
	return true
}

func containsToken(tokens []string, token string) bool {
	for _, current := range tokens {
		if current == token {
			return true
		}
	}
	return false
}

func splitSelectorList(input, pseudoName string) ([]string, error) {
	text := strings.TrimSpace(input)
	errorPrefix := fmt.Sprintf("invalid :%s() pseudo-class selector `%%s`", pseudoName)
	if text == "" {
		return nil, fmt.Errorf(errorPrefix, input)
	}

	parts := make([]string, 0, 2)
	depth := 0
	start := 0
	for i := 0; i < len(text); i++ {
		switch text[i] {
		case '(':
			depth++
		case ')':
			if depth > 0 {
				depth--
			}
		case ',':
			if depth == 0 {
				part := strings.TrimSpace(text[start:i])
				if part == "" {
					return nil, fmt.Errorf(errorPrefix, input)
				}
				parts = append(parts, part)
				start = i + 1
			}
		}
	}

	part := strings.TrimSpace(text[start:])
	if part == "" {
		return nil, fmt.Errorf(errorPrefix, input)
	}
	parts = append(parts, part)
	return parts, nil
}

func isSelectorNameStart(ch byte) bool {
	return (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z') || ch == '_'
}

func isSelectorNameChar(ch byte) bool {
	return isSelectorNameStart(ch) ||
		(ch >= '0' && ch <= '9') ||
		ch == '-'
}

func isCustomStateIdentifier(value string) bool {
	if value == "" {
		return false
	}
	if value[0] == '-' {
		if len(value) == 1 {
			return false
		}
		if !isSelectorNameStart(value[1]) && value[1] != '-' {
			return false
		}
		for i := 2; i < len(value); i++ {
			if !isSelectorNameChar(value[i]) {
				return false
			}
		}
		return true
	}
	if !isSelectorNameStart(value[0]) {
		return false
	}
	for i := 1; i < len(value); i++ {
		if !isSelectorNameChar(value[i]) {
			return false
		}
	}
	return true
}

func previousElementSibling(store *Store, node *Node) *Node {
	if store == nil || node == nil || node.Parent == 0 {
		return nil
	}
	parent := store.Node(node.Parent)
	if parent == nil {
		return nil
	}
	children := parent.Children
	for i := len(children) - 1; i >= 0; i-- {
		if children[i] == node.ID {
			for j := i - 1; j >= 0; j-- {
				sibling := store.Node(children[j])
				if sibling != nil && sibling.Kind == NodeKindElement {
					return sibling
				}
			}
			return nil
		}
	}
	return nil
}

func previousMatchingSibling(store *Store, node *Node, compound simpleSelector, scopeNodeID NodeID) *Node {
	if store == nil || node == nil || node.Parent == 0 {
		return nil
	}
	parent := store.Node(node.Parent)
	if parent == nil {
		return nil
	}
	children := parent.Children
	for i := len(children) - 1; i >= 0; i-- {
		if children[i] == node.ID {
			for j := i - 1; j >= 0; j-- {
				sibling := store.Node(children[j])
				if sibling == nil || sibling.Kind != NodeKindElement {
					continue
				}
				if compound.matchesWithScope(store, sibling, scopeNodeID) {
					return sibling
				}
			}
			return nil
		}
	}
	return nil
}
