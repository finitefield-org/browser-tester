package dom

type NodeList struct {
	ids []NodeID
}

func newNodeList(ids []NodeID) NodeList {
	if len(ids) == 0 {
		return NodeList{ids: []NodeID{}}
	}
	out := make([]NodeID, len(ids))
	copy(out, ids)
	return NodeList{ids: out}
}

func (l NodeList) Length() int {
	return len(l.ids)
}

func (l NodeList) Item(index int) (NodeID, bool) {
	if index < 0 || index >= len(l.ids) {
		return 0, false
	}
	return l.ids[index], true
}

func (l NodeList) IDs() []NodeID {
	if len(l.ids) == 0 {
		return []NodeID{}
	}
	out := make([]NodeID, len(l.ids))
	copy(out, l.ids)
	return out
}

type collectionKind uint8

const (
	collectionKindChildren collectionKind = iota
	collectionKindScripts
	collectionKindImages
	collectionKindForms
	collectionKindFormElements
	collectionKindSelectedOptions
	collectionKindOptions
	collectionKindTableCells
	collectionKindTableBodies
	collectionKindTableRows
	collectionKindLinks
	collectionKindAnchors
)

type HTMLCollection struct {
	store    *Store
	parentID NodeID
	kind     collectionKind
}

func newHTMLCollection(store *Store, parentID NodeID) HTMLCollection {
	return HTMLCollection{
		store:    store,
		parentID: parentID,
		kind:     collectionKindChildren,
	}
}

func newScriptCollection(store *Store, parentID NodeID) HTMLCollection {
	return HTMLCollection{
		store:    store,
		parentID: parentID,
		kind:     collectionKindScripts,
	}
}

func newImageCollection(store *Store, parentID NodeID) HTMLCollection {
	return HTMLCollection{
		store:    store,
		parentID: parentID,
		kind:     collectionKindImages,
	}
}

func newFormCollection(store *Store, parentID NodeID) HTMLCollection {
	return HTMLCollection{
		store:    store,
		parentID: parentID,
		kind:     collectionKindForms,
	}
}

func newFormElementsCollection(store *Store, parentID NodeID) HTMLCollection {
	return HTMLCollection{
		store:    store,
		parentID: parentID,
		kind:     collectionKindFormElements,
	}
}

func newSelectedOptionsCollection(store *Store, parentID NodeID) HTMLCollection {
	return HTMLCollection{
		store:    store,
		parentID: parentID,
		kind:     collectionKindSelectedOptions,
	}
}

func newOptionsCollection(store *Store, parentID NodeID) HTMLCollection {
	return HTMLCollection{
		store:    store,
		parentID: parentID,
		kind:     collectionKindOptions,
	}
}

func newTableCellsCollection(store *Store, parentID NodeID) HTMLCollection {
	return HTMLCollection{
		store:    store,
		parentID: parentID,
		kind:     collectionKindTableCells,
	}
}

func newTableBodiesCollection(store *Store, parentID NodeID) HTMLCollection {
	return HTMLCollection{
		store:    store,
		parentID: parentID,
		kind:     collectionKindTableBodies,
	}
}

func newRowsCollection(store *Store, parentID NodeID) HTMLCollection {
	return HTMLCollection{
		store:    store,
		parentID: parentID,
		kind:     collectionKindTableRows,
	}
}

func newLinkCollection(store *Store, parentID NodeID) HTMLCollection {
	return HTMLCollection{
		store:    store,
		parentID: parentID,
		kind:     collectionKindLinks,
	}
}

func newAnchorCollection(store *Store, parentID NodeID) HTMLCollection {
	return HTMLCollection{
		store:    store,
		parentID: parentID,
		kind:     collectionKindAnchors,
	}
}

func (c HTMLCollection) Length() int {
	return len(c.elementIDs())
}

func (c HTMLCollection) Item(index int) (NodeID, bool) {
	ids := c.elementIDs()
	if index < 0 || index >= len(ids) {
		return 0, false
	}
	return ids[index], true
}

func (c HTMLCollection) NamedItem(name string) (NodeID, bool) {
	if name == "" {
		return 0, false
	}
	for _, id := range c.elementIDs() {
		node := c.store.Node(id)
		if node == nil {
			continue
		}
		if attr, ok := attributeValue(node.Attrs, "id"); ok && attr == name {
			return id, true
		}
		if attr, ok := attributeValue(node.Attrs, "name"); ok && attr == name {
			return id, true
		}
	}
	return 0, false
}

func (c HTMLCollection) IDs() []NodeID {
	ids := c.elementIDs()
	if len(ids) == 0 {
		return []NodeID{}
	}
	out := make([]NodeID, len(ids))
	copy(out, ids)
	return out
}

func (c HTMLCollection) elementIDs() []NodeID {
	if c.store == nil {
		return []NodeID{}
	}
	switch c.kind {
	case collectionKindScripts:
		return c.store.descendantElementIDs(c.parentID, func(node *Node) bool {
			return node.TagName == "script"
		})
	case collectionKindImages:
		return c.store.descendantElementIDs(c.parentID, func(node *Node) bool {
			return node.TagName == "img"
		})
	case collectionKindForms:
		return c.store.descendantElementIDs(c.parentID, func(node *Node) bool {
			return node.TagName == "form"
		})
	case collectionKindFormElements:
		return c.store.descendantElementIDs(c.parentID, func(node *Node) bool {
			return isFormListedElement(node)
		})
	case collectionKindSelectedOptions:
		return c.store.descendantElementIDs(c.parentID, func(node *Node) bool {
			if node.TagName != "option" {
				return false
			}
			_, ok := attributeValue(node.Attrs, "selected")
			return ok
		})
	case collectionKindOptions:
		return c.store.descendantElementIDs(c.parentID, func(node *Node) bool {
			return node.TagName == "option"
		})
	case collectionKindTableCells:
		root := c.store.Node(c.parentID)
		if root == nil || root.Kind != NodeKindElement || root.TagName != "tr" {
			return []NodeID{}
		}
		out := make([]NodeID, 0, len(root.Children))
		for _, childID := range root.Children {
			child := c.store.Node(childID)
			if child == nil || child.Kind != NodeKindElement {
				continue
			}
			if child.TagName != "td" && child.TagName != "th" {
				continue
			}
			out = append(out, childID)
		}
		return out
	case collectionKindTableBodies:
		root := c.store.Node(c.parentID)
		if root == nil || root.Kind != NodeKindElement || root.TagName != "table" {
			return []NodeID{}
		}
		out := make([]NodeID, 0, len(root.Children))
		for _, childID := range root.Children {
			child := c.store.Node(childID)
			if child == nil || child.Kind != NodeKindElement || child.TagName != "tbody" {
				continue
			}
			out = append(out, childID)
		}
		return out
	case collectionKindTableRows:
		root := c.store.Node(c.parentID)
		if root == nil || root.Kind != NodeKindElement {
			return []NodeID{}
		}
		switch root.TagName {
		case "thead", "tbody", "tfoot":
			out := make([]NodeID, 0, len(root.Children))
			for _, childID := range root.Children {
				child := c.store.Node(childID)
				if child == nil || child.Kind != NodeKindElement || child.TagName != "tr" {
					continue
				}
				out = append(out, childID)
			}
			return out
		case "table":
			return c.store.descendantElementIDs(c.parentID, func(node *Node) bool {
				if node.TagName != "tr" {
					return false
				}
				parent := c.store.Node(node.Parent)
				if parent == nil {
					return false
				}
				if parent.ID == c.parentID {
					return true
				}
				if parent.Kind != NodeKindElement {
					return false
				}
				switch parent.TagName {
				case "thead", "tbody", "tfoot":
					return parent.Parent == c.parentID
				default:
					return false
				}
			})
		default:
			return []NodeID{}
		}
	case collectionKindLinks:
		return c.store.descendantElementIDs(c.parentID, func(node *Node) bool {
			if node.TagName != "a" && node.TagName != "area" {
				return false
			}
			_, ok := attributeValue(node.Attrs, "href")
			return ok
		})
	case collectionKindAnchors:
		return c.store.descendantElementIDs(c.parentID, func(node *Node) bool {
			if node.TagName != "a" {
				return false
			}
			_, ok := attributeValue(node.Attrs, "name")
			return ok
		})
	default:
		parent := c.store.Node(c.parentID)
		if parent == nil {
			return []NodeID{}
		}
		out := make([]NodeID, 0, len(parent.Children))
		for _, childID := range parent.Children {
			child := c.store.Node(childID)
			if child == nil || child.Kind != NodeKindElement {
				continue
			}
			out = append(out, childID)
		}
		return out
	}
}
