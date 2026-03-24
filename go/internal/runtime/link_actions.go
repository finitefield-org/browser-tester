package runtime

import (
	"net/url"
	"path"
	"strings"

	"browsertester/internal/dom"
)

func (s *Session) applyHyperlinkDefaultAction(node *dom.Node) error {
	if s == nil || node == nil || node.Kind != dom.NodeKindElement {
		return nil
	}

	href, ok := attributeValue(node.Attrs, "href")
	if !ok {
		return nil
	}
	destination := resolveHyperlinkURL(s.URL(), href)
	if strings.TrimSpace(destination) == "" {
		return nil
	}

	if download, ok := attributeValue(node.Attrs, "download"); ok {
		fileName := hyperlinkDownloadFileName(destination, download)
		s.Registry().Downloads().Capture(fileName, []byte(destination))
		return nil
	}

	target, _ := attributeValue(node.Attrs, "target")
	if strings.EqualFold(strings.TrimSpace(target), "_blank") {
		return s.Registry().Open().Invoke(destination)
	}

	return s.Navigate(destination)
}

func resolveHyperlinkURL(baseURL, href string) string {
	href = strings.TrimSpace(href)
	if href == "" {
		return ""
	}

	baseURL = strings.TrimSpace(baseURL)
	if baseURL == "" {
		return href
	}

	base, err := url.Parse(baseURL)
	if err != nil {
		return href
	}
	ref, err := url.Parse(href)
	if err != nil {
		return href
	}
	return base.ResolveReference(ref).String()
}

func hyperlinkDownloadFileName(destination, suggested string) string {
	suggested = strings.TrimSpace(suggested)
	if suggested != "" {
		return suggested
	}

	parsed, err := url.Parse(destination)
	if err != nil {
		return "download"
	}
	name := path.Base(parsed.Path)
	if name == "." || name == "/" || name == "" {
		return "download"
	}
	return name
}
