package runtime

func (s *Session) AsideCount() int {
	return s.elementCountForSelector("aside")
}
