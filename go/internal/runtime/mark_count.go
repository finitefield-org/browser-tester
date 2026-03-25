package runtime

func (s *Session) MarkCount() int {
	return s.elementCountForSelector("mark")
}
