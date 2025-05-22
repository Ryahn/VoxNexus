import { defineStore } from 'pinia';
import axios from 'axios';

export const useAuthStore = defineStore('auth', {
    state: () => ({
        user: null,
        accessToken: null,
        refreshToken: null
    }),

    getters: {
        isAuthenticated: (state) => !!state.accessToken,
        currentUser: (state) => state.user
    },

    actions: {
        async login(email, password) {
            try {
                const response = await axios.post('/api/auth/login', {
                    email,
                    password
                });

                this.user = response.data.user;
                this.accessToken = response.data.accessToken;
                this.refreshToken = response.data.refreshToken;

                // Set default authorization header for future requests
                axios.defaults.headers.common['Authorization'] = `Bearer ${this.accessToken}`;

                return response.data;
            } catch (error) {
                console.error('Login error:', error);
                throw error;
            }
        },

        async register(username, email, password) {
            try {
                const response = await axios.post('/api/auth/register', {
                    username,
                    email,
                    password,
                    confirmPassword: password
                });

                this.user = response.data.user;
                this.accessToken = response.data.accessToken;
                this.refreshToken = response.data.refreshToken;

                // Set default authorization header for future requests
                axios.defaults.headers.common['Authorization'] = `Bearer ${this.accessToken}`;

                return response.data;
            } catch (error) {
                console.error('Registration error:', error);
                throw error;
            }
        },

        async logout() {
            try {
                if (this.refreshToken) {
                    await axios.post('/api/auth/logout', {
                        refreshToken: this.refreshToken
                    });
                }

                this.user = null;
                this.accessToken = null;
                this.refreshToken = null;
                delete axios.defaults.headers.common['Authorization'];
            } catch (error) {
                console.error('Logout error:', error);
                throw error;
            }
        },

        async refreshAccessToken() {
            try {
                const response = await axios.post('/api/auth/refresh-token', {
                    refreshToken: this.refreshToken
                });

                this.accessToken = response.data.accessToken;
                this.refreshToken = response.data.refreshToken;

                // Update authorization header
                axios.defaults.headers.common['Authorization'] = `Bearer ${this.accessToken}`;

                return response.data;
            } catch (error) {
                console.error('Token refresh error:', error);
                throw error;
            }
        },

        // Configure axios defaults
        configureAxios() {
            axios.defaults.baseURL = 'http://localhost:5000';
            axios.defaults.withCredentials = true;
        },

        // Add axios interceptor for token refresh
        addAxiosInterceptor() {
            axios.interceptors.response.use(
                (response) => response,
                async (error) => {
                    const originalRequest = error.config;
                    
                    if (error.response?.status === 401 && !originalRequest._retry) {
                        originalRequest._retry = true;
                        
                        try {
                            const response = await this.refreshAccessToken();
                            
                            // Retry original request
                            originalRequest.headers['Authorization'] = `Bearer ${response.accessToken}`;
                            return axios(originalRequest);
                        } catch (error) {
                            // Refresh token failed, logout
                            await this.logout();
                            return Promise.reject(error);
                        }
                    }
                    
                    return Promise.reject(error);
                }
            );
        },

        // Update axios headers when token changes
        updateAxiosHeaders() {
            if (this.accessToken) {
                axios.defaults.headers.common['Authorization'] = `Bearer ${this.accessToken}`;
            } else {
                delete axios.defaults.headers.common['Authorization'];
            }
        },

        // Initialize headers
        initializeHeaders() {
            this.updateAxiosHeaders();
            this.addAxiosInterceptor();
        },

        async logoutAll() {
            try {
                await axios.post('/api/auth/logout-all');
                this.user = null;
                this.accessToken = null;
                this.refreshToken = null;
                delete axios.defaults.headers.common['Authorization'];
            } catch (error) {
                console.error('Logout all error:', error);
                throw error;
            }
        },

        async fetchCurrentUser() {
            try {
                const response = await axios.get('/api/auth/me');
                this.user = response.data.user;
                return this.user;
            } catch (error) {
                console.error('Fetch current user error:', error);
                throw error;
            }
        },

        async updateProfile(profileData) {
            try {
                const response = await axios.put('/api/auth/profile', profileData);
                this.user = response.data.user;
                return this.user;
            } catch (error) {
                console.error('Update profile error:', error);
                throw error;
            }
        },

        async updatePassword(currentPassword, newPassword) {
            try {
                await axios.put('/api/auth/password', {
                    currentPassword,
                    newPassword
                });
            } catch (error) {
                console.error('Update password error:', error);
                throw error;
            }
        },

        async getSessions() {
            try {
                const response = await axios.get('/api/auth/sessions');
                return response.data.sessions;
            } catch (error) {
                console.error('Get sessions error:', error);
                throw error;
            }
        },

        async logoutSession(sessionId) {
            try {
                await axios.delete(`/api/auth/sessions/${sessionId}`);
            } catch (error) {
                console.error('Logout session error:', error);
                throw error;
            }
        },

        async uploadAvatar(formData) {
            try {
                const response = await axios.post('/api/auth/avatar', formData, {
                    headers: {
                        'Content-Type': 'multipart/form-data'
                    }
                });
                
                if (this.user) {
                    this.user.avatar = response.data.avatar;
                }
                
                return response.data.avatar;
            } catch (error) {
                console.error('Upload avatar error:', error);
                throw error;
            }
        },

        async uploadBanner(formData) {
            try {
                const response = await axios.post('/api/auth/banner', formData, {
                    headers: {
                        'Content-Type': 'multipart/form-data'
                    }
                });
                
                if (this.user) {
                    this.user.banner = response.data.banner;
                }
                
                return response.data.banner;
            } catch (error) {
                console.error('Upload banner error:', error);
                throw error;
            }
        }
    }
}); 