package dom

type NodeID int64

type NodeKind uint8

const (
	NodeKindDocument NodeKind = iota
	NodeKindElement
	NodeKindText
)

type Attribute struct {
	Name     string
	Value    string
	HasValue bool
}

type Node struct {
	ID       NodeID
	Kind     NodeKind
	Parent   NodeID
	Children []NodeID

	TagName string
	Attrs   []Attribute
	Text    string

	DefaultAttrs []Attribute
	DefaultText  string
	UserValidity bool
}

type Store struct {
	nodes         map[NodeID]*Node
	documentID    NodeID
	focusedNodeID NodeID
	targetNodeID  NodeID
	currentURL    string
	visitedURLs   map[string]struct{}
	nextNodeID    NodeID
	sourceHTML    string
}

func NewStore() *Store {
	s := &Store{}
	s.Reset()
	return s
}

func NewEmpty() *Store {
	return NewStore()
}

func (s *Store) Reset() {
	s.nodes = map[NodeID]*Node{}
	s.sourceHTML = ""
	s.focusedNodeID = 0
	s.targetNodeID = 0
	s.currentURL = ""
	s.visitedURLs = map[string]struct{}{}
	s.nextNodeID = 1
	s.documentID = s.newNode(Node{
		Kind: NodeKindDocument,
	})
}

func (s *Store) CurrentURL() string {
	if s == nil {
		return ""
	}
	return s.currentURL
}

func (s *Store) DocumentID() NodeID {
	if s == nil {
		return 0
	}
	return s.documentID
}

func (s *Store) SourceHTML() string {
	if s == nil {
		return ""
	}
	return s.sourceHTML
}

func (s *Store) NodeCount() int {
	if s == nil {
		return 0
	}
	return len(s.nodes)
}

func (s *Store) Node(id NodeID) *Node {
	if s == nil {
		return nil
	}
	return s.nodes[id]
}

func (s *Store) Nodes() []*Node {
	if s == nil {
		return nil
	}
	out := make([]*Node, 0, len(s.nodes))
	for id := NodeID(1); id < s.nextNodeID; id++ {
		node := s.nodes[id]
		if node != nil {
			out = append(out, node)
		}
	}
	return out
}

func (s *Store) newNode(seed Node) NodeID {
	if s.nodes == nil {
		s.nodes = map[NodeID]*Node{}
	}
	id := s.nextNodeID
	s.nextNodeID++
	node := seed
	node.ID = id
	if node.Children == nil {
		node.Children = []NodeID{}
	}
	if node.Attrs == nil {
		node.Attrs = []Attribute{}
	}
	s.nodes[id] = &node
	return id
}

func (s *Store) appendChild(parentID, childID NodeID) {
	parent := s.nodes[parentID]
	child := s.nodes[childID]
	if parent == nil || child == nil {
		return
	}
	child.Parent = parentID
	parent.Children = append(parent.Children, childID)
}

func (s *Store) documentChildren() []NodeID {
	if s == nil {
		return nil
	}
	document := s.nodes[s.documentID]
	if document == nil {
		return nil
	}
	return document.Children
}

func (s *Store) descendantElementIDs(rootID NodeID, predicate func(*Node) bool) []NodeID {
	if s == nil {
		return []NodeID{}
	}
	root := s.Node(rootID)
	if root == nil {
		return []NodeID{}
	}
	out := make([]NodeID, 0)
	var visit func(NodeID)
	visit = func(parentID NodeID) {
		parent := s.Node(parentID)
		if parent == nil {
			return
		}
		for _, childID := range parent.Children {
			child := s.Node(childID)
			if child == nil {
				continue
			}
			if child.Kind == NodeKindElement && predicate != nil && predicate(child) {
				out = append(out, childID)
			}
			visit(childID)
		}
	}
	visit(rootID)
	return out
}
