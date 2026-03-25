package runtime

func (s *Session) FocusedNodeID() int64 {
	if s == nil || s.domStore == nil {
		return 0
	}
	return int64(s.domStore.FocusedNodeID())
}
