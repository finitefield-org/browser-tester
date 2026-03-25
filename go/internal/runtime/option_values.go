package runtime

type optionValueRecord struct {
	NodeID int64
	Value  string
}

func (s *Session) OptionValues() []optionValueRecord {
	return s.optionValuesForSelector("option")
}

func (s *Session) SelectedOptionValues() []optionValueRecord {
	return s.optionValuesForSelector("option[selected]")
}

func (s *Session) optionValuesForSelector(selector string) []optionValueRecord {
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
	out := make([]optionValueRecord, 0, nodes.Length())
	for i := 0; i < nodes.Length(); i++ {
		nodeID, ok := nodes.Item(i)
		if !ok {
			continue
		}
		out = append(out, optionValueRecord{
			NodeID: int64(nodeID),
			Value:  store.ValueForNode(nodeID),
		})
	}
	return out
}
