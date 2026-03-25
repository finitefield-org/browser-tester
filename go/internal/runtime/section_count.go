package runtime

func (s *Session) SectionCount() int {
	return s.elementCountForSelector("section")
}
