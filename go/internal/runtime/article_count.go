package runtime

func (s *Session) ArticleCount() int {
	return s.elementCountForSelector("article")
}
