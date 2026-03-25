package runtime

type optgroupLabelRecord struct {
	NodeID int64
	Label  string
}

func (s *Session) OptgroupLabels() []optgroupLabelRecord {
	return s.optgroupLabelsForSelector("optgroup")
}

func (s *Session) optgroupLabelsForSelector(selector string) []optgroupLabelRecord {
	if s == nil {
		return nil
	}
	store, err := s.ensureDOM()
	if err != nil || store == nil {
		return nil
	}
	nodes, err := store.QuerySelectorAll(selector)
	if err != nil {
		return nil
	}
	out := make([]optgroupLabelRecord, 0, nodes.Length())
	for i := 0; i < nodes.Length(); i++ {
		nodeID, ok := nodes.Item(i)
		if !ok {
			continue
		}
		out = append(out, optgroupLabelRecord{
			NodeID: int64(nodeID),
			Label:  store.OptgroupLabelForNode(nodeID),
		})
	}
	return out
}
