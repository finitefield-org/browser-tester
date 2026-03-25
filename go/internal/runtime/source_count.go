package runtime

func (s *Session) SourceCount() int {
	return s.elementCountForSelector("source")
}
