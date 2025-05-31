const ZINC_URL = process.env.ZINC_SEARCH_URL || 'http://localhost:4080'
const ZINC_USER = process.env.ZINC_FIRST_ADMIN_USER || 'admin'
const ZINC_PASS = process.env.ZINC_FIRST_ADMIN_PASSWORD || 'admin'
const ZINC_INDEX = 'users'
const ZINC_SERVER_INDEX = 'servers'
const ZINC_MESSAGE_INDEX = 'messages'

async function zincRequest(path: string, options: RequestInit) {
  const url = `${ZINC_URL}/api${path}`
  const headers = options.headers || {}
  return fetch(url, {
    ...options,
    headers: {
      ...headers,
      'Content-Type': 'application/json',
      'Authorization': 'Basic ' + Buffer.from(`${ZINC_USER}:${ZINC_PASS}`).toString('base64'),
    },
  })
}

export async function indexUser(user: { id: string, username: string, email: string, avatarUrl?: string, bio?: string }) {
  return zincRequest(`/index/${ZINC_INDEX}/_doc`, {
    method: 'POST',
    body: JSON.stringify({
      id: user.id,
      username: user.username,
      email: user.email,
      avatarUrl: user.avatarUrl,
      bio: user.bio,
    }),
  })
}

export async function searchUsers(query: string, limit = 20) {
  const body = {
    search_type: 'match',
    query: {
      term: query,
      fields: ['username', 'email', 'bio'],
    },
    max_results: limit,
  }
  const res = await zincRequest(`/search/${ZINC_INDEX}`, {
    method: 'POST',
    body: JSON.stringify(body),
  })
  if (!res.ok) throw new Error('ZincSearch error')
  const data = await res.json()
  return data.hits?.hits?.map((hit: any) => hit._source) || []
}

export async function removeUserFromIndex(userId: string) {
  return zincRequest(`/index/${ZINC_INDEX}/_doc/${userId}`, {
    method: 'DELETE',
  });
}

export async function indexServer(server: { id: string, name: string, ownerId: string, createdAt: string }) {
  return zincRequest(`/index/${ZINC_SERVER_INDEX}/_doc`, {
    method: 'POST',
    body: JSON.stringify(server),
  });
}

export async function removeServerFromIndex(serverId: string) {
  return zincRequest(`/index/${ZINC_SERVER_INDEX}/_doc/${serverId}`, {
    method: 'DELETE',
  });
}

export async function indexMessage(message: { id: string, content: string, authorId: string, channelId: string, createdAt: string }) {
  return zincRequest(`/index/${ZINC_MESSAGE_INDEX}/_doc`, {
    method: 'POST',
    body: JSON.stringify(message),
  });
}

export async function removeMessageFromIndex(messageId: string) {
  return zincRequest(`/index/${ZINC_MESSAGE_INDEX}/_doc/${messageId}`, {
    method: 'DELETE',
  });
}

export async function searchMessages(query: string, channelId: string, limit = 20) {
  const body = {
    search_type: 'match',
    query: {
      term: query,
      fields: ['content'],
      filter: [{ term: channelId, field: 'channelId' }],
    },
    max_results: limit,
  };
  const res = await zincRequest(`/search/${ZINC_MESSAGE_INDEX}`, {
    method: 'POST',
    body: JSON.stringify(body),
  });
  if (!res.ok) throw new Error('ZincSearch error');
  const data = await res.json();
  return data.hits?.hits?.map((hit: any) => hit._source) || [];
}

export async function searchDirectMessages(query: string, userAId: string, userBId: string, limit = 20) {
  const body = {
    search_type: 'match',
    query: {
      term: query,
      fields: ['content'],
      filter: [
        {
          or: [
            { and: [ { term: userAId, field: 'from' }, { term: userBId, field: 'to' } ] },
            { and: [ { term: userBId, field: 'from' }, { term: userAId, field: 'to' } ] },
          ]
        }
      ],
    },
    max_results: limit,
  };
  const res = await zincRequest(`/search/directmessages`, {
    method: 'POST',
    body: JSON.stringify(body),
  });
  if (!res.ok) throw new Error('ZincSearch error');
  const data = await res.json();
  return data.hits?.hits?.map((hit: any) => hit._source) || [];
} 