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

type HTMLCollection struct {
	store    *Store
	parentID NodeID
}

func newHTMLCollection(store *Store, parentID NodeID) HTMLCollection {
	return HTMLCollection{
		store:    store,
		parentID: parentID,
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
