package runtime

import (
	"strings"

	"browsertester/internal/dom"
)

func (s *Session) LastInlineScriptHTML() string {
	if s == nil {
		return ""
	}
	if s.lastInlineScriptHTML != "" || s.domReady {
		return s.lastInlineScriptHTML
	}
	if strings.TrimSpace(s.config.HTML) == "" {
		return ""
	}
	store := dom.NewStore()
	if err := store.BootstrapHTML(s.config.HTML); err != nil {
		return ""
	}
	nodes := store.Nodes()
	for i := len(nodes) - 1; i >= 0; i-- {
		node := nodes[i]
		if node == nil || node.Kind != dom.NodeKindElement || node.TagName != "script" {
			continue
		}
		outerHTML, err := store.OuterHTMLForNode(node.ID)
		if err != nil {
			return ""
		}
		return outerHTML
	}
	return ""
}
