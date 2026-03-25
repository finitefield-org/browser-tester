package runtime

type optionLabelRecord struct {
	NodeID int64
	Label  string
}

func (s *Session) OptionLabels() []optionLabelRecord {
	return s.optionLabelsForSelector("option")
}

func (s *Session) SelectedOptionLabels() []optionLabelRecord {
	return s.optionLabelsForSelector("option[selected]")
}

func (s *Session) optionLabelsForSelector(selector string) []optionLabelRecord {
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
	out := make([]optionLabelRecord, 0, nodes.Length())
	for i := 0; i < nodes.Length(); i++ {
		nodeID, ok := nodes.Item(i)
		if !ok {
			continue
		}
		out = append(out, optionLabelRecord{
			NodeID: int64(nodeID),
			Label:  store.OptionLabelForNode(nodeID),
		})
	}
	return out
}
