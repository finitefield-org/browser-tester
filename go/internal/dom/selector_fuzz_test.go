package dom

import "testing"

func FuzzSelectorQueries(f *testing.F) {
	seeds := []string{
		"",
		"div",
		"#nav",
		"a[href]",
		"main > section",
		":has(a)",
		":not(.missing)",
		":nth-child(2n+1)",
		":is(a, button)",
		"a:local-link",
	}
	for _, seed := range seeds {
		f.Add(seed)
	}

	var store Store
	if err := store.BootstrapHTML(`<main id="root"><section id="panel" class="alpha beta" lang="en"><a id="nav" href="/next">Go</a><input id="name" type="text" placeholder="Name"><textarea id="story" placeholder="Story"></textarea><button id="submit" type="submit">Save</button><div id="empty"></div></section></main>`); err != nil {
		panic(err)
	}

	anchorID, ok, err := store.QuerySelector("#nav")
	if err != nil || !ok {
		panic("seed document is missing #nav")
	}

	f.Fuzz(func(t *testing.T, selector string) {
		nodes, err := store.QuerySelectorAll(selector)
		if err == nil {
			seen := make(map[NodeID]struct{}, nodes.Length())
			for i := 0; i < nodes.Length(); i++ {
				id, ok := nodes.Item(i)
				if !ok {
					t.Fatalf("QuerySelectorAll(%q).Item(%d) = false", selector, i)
				}
				if _, exists := seen[id]; exists {
					t.Fatalf("QuerySelectorAll(%q) returned duplicate node %d", selector, id)
				}
				seen[id] = struct{}{}
			}
		}

		if id, ok, err := store.QuerySelector(selector); err == nil && ok {
			matched, err := store.Matches(id, selector)
			if err != nil {
				t.Fatalf("Matches(QuerySelector(%q)) error = %v", selector, err)
			}
			if !matched {
				t.Fatalf("Matches(QuerySelector(%q)) = false", selector)
			}
		}

		if id, ok, err := store.Closest(anchorID, selector); err == nil && ok {
			matched, err := store.Matches(id, selector)
			if err != nil {
				t.Fatalf("Matches(Closest(%q)) error = %v", selector, err)
			}
			if !matched {
				t.Fatalf("Matches(Closest(%q)) = false", selector)
			}
		}
	})
}
