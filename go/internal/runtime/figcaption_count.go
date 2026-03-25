package runtime

func (s *Session) FigcaptionCount() int {
	return s.elementCountForSelector("figcaption")
}
