package runtime

func (s *Session) DataCount() int {
	return s.elementCountForSelector("data")
}
