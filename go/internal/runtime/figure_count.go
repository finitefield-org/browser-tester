package runtime

func (s *Session) FigureCount() int {
	return s.elementCountForSelector("figure")
}
