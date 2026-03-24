package dom

import (
	"fmt"
	"strings"
)

type Dataset struct {
	store  *Store
	nodeID NodeID
}

func (s *Store) Dataset(nodeID NodeID) (Dataset, error) {
	if s == nil {
		return Dataset{}, fmt.Errorf("dom store is nil")
	}
	if _, err := s.elementNode(nodeID); err != nil {
		return Dataset{}, err
	}
	return Dataset{store: s, nodeID: nodeID}, nil
}

func (d Dataset) Values() map[string]string {
	node, err := d.element()
	if err != nil {
		return map[string]string{}
	}
	out := map[string]string{}
	for _, attr := range node.Attrs {
		name, ok := dataAttrToDatasetName(attr.Name)
		if !ok {
			continue
		}
		out[name] = attr.Value
	}
	return out
}

func (d Dataset) Get(name string) (string, bool) {
	attrName, ok := tryDatasetNameToDataAttr(name)
	if !ok {
		return "", false
	}
	node, err := d.element()
	if err != nil {
		return "", false
	}
	value, ok := attributeValue(node.Attrs, attrName)
	return value, ok
}

func (d Dataset) Set(name, value string) error {
	if d.store == nil {
		return fmt.Errorf("dom store is nil")
	}
	attrName, err := datasetNameToDataAttr(name)
	if err != nil {
		return err
	}
	return d.store.SetAttribute(d.nodeID, attrName, value)
}

func (d Dataset) Remove(name string) error {
	if d.store == nil {
		return fmt.Errorf("dom store is nil")
	}
	attrName, err := datasetNameToDataAttr(name)
	if err != nil {
		return err
	}
	return d.store.RemoveAttribute(d.nodeID, attrName)
}

func (d Dataset) element() (*Node, error) {
	if d.store == nil {
		return nil, fmt.Errorf("dom store is nil")
	}
	return d.store.elementNode(d.nodeID)
}

func dataAttrToDatasetName(attrName string) (string, bool) {
	if !strings.HasPrefix(attrName, "data-") {
		return "", false
	}
	rest := attrName[len("data-"):]
	if rest == "" {
		return "", false
	}
	for i := 0; i < len(rest); i++ {
		if rest[i] >= 'A' && rest[i] <= 'Z' {
			return "", false
		}
	}

	// Hyphenated names become camel-cased:
	// remove '-' that is followed by an ASCII lower alpha and uppercase that alpha.
	var b strings.Builder
	b.Grow(len(rest))
	for i := 0; i < len(rest); i++ {
		ch := rest[i]
		if ch == '-' && i+1 < len(rest) {
			next := rest[i+1]
			if next >= 'a' && next <= 'z' {
				b.WriteByte(next - ('a' - 'A'))
				i++
				continue
			}
		}
		b.WriteByte(ch)
	}
	return b.String(), true
}

func tryDatasetNameToDataAttr(name string) (string, bool) {
	attrName, err := datasetNameToDataAttr(name)
	if err != nil {
		return "", false
	}
	return attrName, true
}

func datasetNameToDataAttr(name string) (string, error) {
	normalized := strings.TrimSpace(name)
	if normalized == "" {
		return "", fmt.Errorf("dataset name must not be empty")
	}

	// Reject hyphen-minus followed by an ASCII lower alpha.
	for i := 0; i+1 < len(normalized); i++ {
		if normalized[i] == '-' {
			next := normalized[i+1]
			if next >= 'a' && next <= 'z' {
				return "", fmt.Errorf("invalid dataset name: %q", name)
			}
		}
	}

	var b strings.Builder
	b.Grow(len(normalized) + len("data-") + 4)
	b.WriteString("data-")
	for i := 0; i < len(normalized); i++ {
		ch := normalized[i]
		if ch >= 'A' && ch <= 'Z' {
			b.WriteByte('-')
			b.WriteByte(ch + ('a' - 'A'))
			continue
		}
		b.WriteByte(ch)
	}

	out := b.String()
	for i := 0; i < len(out); i++ {
		if !isAttrNameChar(out[i]) {
			return "", fmt.Errorf("invalid dataset name: %q", name)
		}
		// The derived attribute name must not contain uppercase.
		if out[i] >= 'A' && out[i] <= 'Z' {
			return "", fmt.Errorf("invalid dataset name: %q", name)
		}
	}
	return out, nil
}
