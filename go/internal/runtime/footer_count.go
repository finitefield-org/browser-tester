package runtime

func (s *Session) FooterCount() int {
	return s.elementCountForSelector("footer")
}
