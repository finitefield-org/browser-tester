package runtime

func (s *Session) ParagraphCount() int {
	return s.elementCountForSelector("p")
}
