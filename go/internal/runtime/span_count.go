package runtime

func (s *Session) SpanCount() int {
	return s.elementCountForSelector("span")
}
