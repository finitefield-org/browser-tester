package runtime

func (s *Session) BlockquoteCount() int {
	return s.elementCountForSelector("blockquote")
}
