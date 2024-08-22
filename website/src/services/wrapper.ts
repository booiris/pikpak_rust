import { Configuration } from '@/api'
import { useBackendUrlStore } from '@/stores/backend_url'
import { useTokenStore } from '@/stores/token'

class clientWrapperFactory<C> {
    now_url: string
    client: C
    c_constructor: new (config: Configuration) => C
    constructor(c_constructor: new (config: Configuration) => C) {
        this.now_url = useBackendUrlStore().storedUrl
        this.client = new c_constructor(
            new Configuration({
                basePath: useBackendUrlStore().storedUrl,
                accessToken: useTokenStore().storedToken
            })
        )
        this.c_constructor = c_constructor
    }
    public get_client(): C {
        if (useBackendUrlStore().storedUrl == this.now_url) {
            return this.client
        } else {
            console.log('url changed')
            this.now_url = useBackendUrlStore().storedUrl
            this.client = new this.c_constructor(
                new Configuration({
                    basePath: useBackendUrlStore().storedUrl,
                    accessToken: useTokenStore().storedToken
                })
            )
            return this.client
        }
    }
}

export function clientWrapper<C>(t: new (config: Configuration) => C): () => C {
    const f = new clientWrapperFactory(t)
    return () => {
        return f.get_client()
    }
}
