import { models } from '../libs/database/db.js';
import crypto from 'crypto';
import AttachmentService from '../services/AttachmentService.js';
import { utility } from '../libs/utils.js';

const MAX_ICON_SIZE = {
    width: 512,
    height: 512
}

const MAX_BANNER_SIZE = {
    width: 2000,
    height: 1000
}

/**====================================
 * Server Methods
 ======================================*/
export const createServer = async (req, res) => {
    try {
        const { name, icon, banner, description, isPublic, isNsfw, type } = req.body;
        const userId = req.user.id; // Assuming user is authenticated

        // Validate required fields
        if (!name) {
            return res.status(400).json({ error: 'Server name is required' });
        }

        // Process icon if provided
        let iconUrl = null;
        if (icon) {
            const iconResult = await AttachmentService.processImage(icon, {
                maxWidth: MAX_ICON_SIZE.width,
                maxHeight: MAX_ICON_SIZE.height
            });
            iconUrl = iconResult.url;
        }

        // Process banner if provided
        let bannerUrl = null;
        if (banner) {
            const bannerResult = await AttachmentService.processImage(banner, {
                maxWidth: MAX_BANNER_SIZE.width,
                maxHeight: MAX_BANNER_SIZE.height
            });
            bannerUrl = bannerResult.url;
        }

        // Create server with default channels
        const server = await models.Server.create({
            name,
            icon: iconUrl,
            banner: bannerUrl,
            description,
            isPublic: isPublic ?? true,
            isNsfw: isNsfw ?? false,
            type: type ?? 'community',
            ownerId: userId,
            members: [{
                userId,
                roles: ['owner'],
                joinedAt: new Date()
            }],
            channels: [
                {
                    name: 'general',
                    type: 'text',
                    position: 0
                },
                {
                    name: 'voice',
                    type: 'voice',
                    position: 1
                }
            ]
        });

        return res.status(201).json(server);
    } catch (error) {
        console.error('Server creation error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

export const getServers = async (req, res) => {
    try {
        const servers = await models.Server.find({});
        return res.status(200).json(servers);
    } catch (error) {
        console.error('Server retrieval error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

export const getServer = async (req, res) => {
    try {
        const server = await models.Server.findById(req.params.id);
        return res.status(200).json(server);
    } catch (error) {
        console.error('Server retrieval error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

export const updateServer = async (req, res) => {
    try {
        const server = await models.Server.findByIdAndUpdate(req.params.id, req.body, { new: true });
        return res.status(200).json(server);
    } catch (error) {
        console.error('Server update error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

export const deleteServer = async (req, res) => {
    try {
        await models.Server.findByIdAndDelete(req.params.id);
        return res.status(200).json({ message: 'Server deleted successfully' });
    } catch (error) {
        console.error('Server deletion error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

export const joinServer = async (req, res) => {
    try {
        const { inviteCode } = req.body;
        const userId = req.user.id;

        // Find server by invite code
        const server = await models.Server.findOne({ 'invites.code': inviteCode });
        if (!server) {
            return res.status(404).json({ error: 'Invalid invite code' });
        }

        // Check if user is already a member
        const isMember = server.members.some(member => member.userId.toString() === userId);
        if (isMember) {
            return res.status(400).json({ error: 'Already a member of this server' });
        }

        // Add user to server members
        server.members.push({
            userId,
            roles: ['member'],
            joinedAt: new Date()
        });

        await server.save();
        return res.status(200).json(server);
    } catch (error) {
        console.error('Server join error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

export const leaveServer = async (req, res) => {
    try {
        const serverId = req.params.id;
        const userId = req.user.id;

        const server = await models.Server.findById(serverId);
        if (!server) {
            return res.status(404).json({ error: 'Server not found' });
        }

        // Check if user is the owner
        if (server.ownerId.toString() === userId) {
            return res.status(400).json({ error: 'Server owner cannot leave. Transfer ownership or delete the server.' });
        }

        // Remove user from members
        server.members = server.members.filter(member => member.userId.toString() !== userId);
        await server.save();

        return res.status(200).json({ message: 'Successfully left the server' });
    } catch (error) {
        console.error('Server leave error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}


/**====================================
 * Server Invite Methods
 ======================================*/
export const getServerInvites = async (req, res) => {
    try {
        const server = await models.Server.findById(req.params.id);
        return res.status(200).json(server);
    } catch (error) {
        console.error('Server invites retrieval error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

export const getServerInvite = async (req, res) => {
    try {
        const { inviteCode } = req.params;
        const server = await models.Server.findOne({ 'invites.code': inviteCode });
        
        if (!server) {
            return res.status(404).json({ error: 'Invalid invite code' });
        }

        const invite = server.invites.find(inv => inv.code === inviteCode);
        return res.status(200).json(invite);
    } catch (error) {
        console.error('Server invite retrieval error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

export const createServerInvite = async (req, res) => {
    try {
        const serverId = req.params.id;
        const userId = req.user.id;
        const { maxUses, expiresAt } = req.body;

        const server = await models.Server.findById(serverId);
        if (!server) {
            return res.status(404).json({ error: 'Server not found' });
        }

        // Check if user has permission to create invites
        const member = server.members.find(m => m.userId.toString() === userId);
        if (!member || !member.roles.some(role => ['owner', 'admin', 'moderator'].includes(role))) {
            return res.status(403).json({ error: 'Insufficient permissions' });
        }

        // Generate unique invite code
        const code = crypto.randomBytes(9).toString('hex');
        
        const invite = {
            code,
            createdBy: userId,
            createdAt: new Date(),
            maxUses: maxUses || null,
            expiresAt: expiresAt || null,
            uses: 0
        };

        server.invites.push(invite);
        await server.save();

        return res.status(201).json(invite);
    } catch (error) {
        console.error('Server invite creation error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

export const updateServerInvite = async (req, res) => {
    try {
        const { id, inviteCode } = req.params;
        const { maxUses, expiresAt } = req.body;
        const userId = req.user.id;

        const server = await models.Server.findById(id);
        if (!server) {
            return res.status(404).json({ error: 'Server not found' });
        }

        const invite = server.invites.find(inv => inv.code === inviteCode);
        if (!invite) {
            return res.status(404).json({ error: 'Invite not found' });
        }

        // Check if user has permission to update invites
        const member = server.members.find(m => m.userId.toString() === userId);
        if (!member || !member.roles.some(role => ['owner', 'admin'].includes(role))) {
            return res.status(403).json({ error: 'Insufficient permissions' });
        }

        if (maxUses !== undefined) invite.maxUses = maxUses;
        if (expiresAt !== undefined) invite.expiresAt = expiresAt;

        await server.save();
        return res.status(200).json(invite);
    } catch (error) {
        console.error('Server invite update error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

export const deleteServerInvite = async (req, res) => {
    try {
        const { id, inviteCode } = req.params;
        const userId = req.user.id;

        const server = await models.Server.findById(id);
        if (!server) {
            return res.status(404).json({ error: 'Server not found' });
        }

        // Check if user has permission to delete invites
        const member = server.members.find(m => m.userId.toString() === userId);
        if (!member || !member.roles.some(role => ['owner', 'admin'].includes(role))) {
            return res.status(403).json({ error: 'Insufficient permissions' });
        }

        server.invites = server.invites.filter(inv => inv.code !== inviteCode);
        await server.save();

        return res.status(200).json({ message: 'Invite deleted successfully' });
    } catch (error) {
        console.error('Server invite deletion error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

export const getServerInviteMembers = async (req, res) => {

}

export const getServerInviteMember = async (req, res) => {

}

export const updateServerInviteMember = async (req, res) => {

}

export const deleteServerInviteMember = async (req, res) => {

}


/**====================================
 * Server Member Methods
 ======================================*/

export const getServerMembers = async (req, res) => {
    try {
        const server = await models.Server.findById(req.params.id);
        return res.status(200).json(server);
    } catch (error) {
        console.error('Server members retrieval error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

export const getServerMember = async (req, res) => {
    try {
        const server = await models.Server.findById(req.params.id);
        return res.status(200).json(server);
    } catch (error) {
        console.error('Server member retrieval error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

export const updateServerMember = async (req, res) => {
    try {
        const server = await models.Server.findById(req.params.id);
        return res.status(200).json(server);
    } catch (error) {
        console.error('Server member update error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

export const deleteServerMember = async (req, res) => {
    try {
        const server = await models.Server.findById(req.params.id);
        return res.status(200).json(server);
    } catch (error) {
        console.error('Server member deletion error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

export const getServerMemberRoles = async (req, res) => {

}

export const updateServerMemberRoles = async (req, res) => {

}

export const getServerMemberPermissions = async (req, res) => {

}

export const updateServerMemberPermissions = async (req, res) => {

}

export const getServerEmojis = async (req, res) => {

}

export const createServerEmoji = async (req, res) => {

}

export const updateServerEmoji = async (req, res) => {

}

export const deleteServerEmoji = async (req, res) => {

}

export const getServerStickers = async (req, res) => {

}

export const createServerSticker = async (req, res) => {

}

export const updateServerSticker = async (req, res) => {

}

export const deleteServerSticker = async (req, res) => {

}

export const getServerBans = async (req, res) => {

}

export const createServerBan = async (req, res) => {

}

export const getServerBan = async (req, res) => {

}

export const updateServerBan = async (req, res) => {

}

export const deleteServerBan = async (req, res) => {

}


/**====================================
 * Server Role Methods
 ======================================*/

export const getServerRoles = async (req, res) => {
    try {
        const server = await models.Server.findById(req.params.id);
        return res.status(200).json(server);
    } catch (error) {
        console.error('Server roles retrieval error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

export const getServerRole = async (req, res) => {
    try {
        const { id, roleId } = req.params;
        const server = await models.Server.findById(id);
        
        if (!server) {
            return res.status(404).json({ error: 'Server not found' });
        }

        const role = server.roles.find(r => r._id.toString() === roleId);
        if (!role) {
            return res.status(404).json({ error: 'Role not found' });
        }

        return res.status(200).json(role);
    } catch (error) {
        console.error('Server role retrieval error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

export const createServerRole = async (req, res) => {
    try {
        const serverId = req.params.id;
        const userId = req.user.id;
        const { name, color, permissions, position } = req.body;

        const server = await models.Server.findById(serverId);
        if (!server) {
            return res.status(404).json({ error: 'Server not found' });
        }

        // Check if user has permission to create roles
        const member = server.members.find(m => m.userId.toString() === userId);
        if (!member || !member.roles.some(role => ['owner', 'admin'].includes(role))) {
            return res.status(403).json({ error: 'Insufficient permissions' });
        }

        const role = {
            id: utility.generateSnowflake(),
            name,
            color: color || '#000000',
            permissions: permissions || [],
            position: position || server.roles.length
        };

        server.roles.push(role);
        await server.save();

        return res.status(201).json(role);
    } catch (error) {
        console.error('Server role creation error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

export const updateServerRole = async (req, res) => {
    try {
        const { id, roleId } = req.params;
        const userId = req.user.id;
        const { name, color, permissions, position } = req.body;

        const server = await models.Server.findById(id);
        if (!server) {
            return res.status(404).json({ error: 'Server not found' });
        }

        const role = server.roles.find(r => r._id.toString() === roleId);
        if (!role) {
            return res.status(404).json({ error: 'Role not found' });
        }

        // Check if user has permission to update roles
        const member = server.members.find(m => m.userId.toString() === userId);
        if (!member || !member.roles.some(role => ['owner', 'admin'].includes(role))) {
            return res.status(403).json({ error: 'Insufficient permissions' });
        }

        if (name !== undefined) role.name = name;
        if (color !== undefined) role.color = color;
        if (permissions !== undefined) role.permissions = permissions;
        if (position !== undefined) role.position = position;

        await server.save();
        return res.status(200).json(role);
    } catch (error) {
        console.error('Server role update error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

export const deleteServerRole = async (req, res) => {
    try {
        const { id, roleId } = req.params;
        const userId = req.user.id;

        const server = await models.Server.findById(id);
        if (!server) {
            return res.status(404).json({ error: 'Server not found' });
        }

        // Check if user has permission to delete roles
        const member = server.members.find(m => m.userId.toString() === userId);
        if (!member || !member.roles.some(role => ['owner', 'admin'].includes(role))) {
            return res.status(403).json({ error: 'Insufficient permissions' });
        }

        // Remove role from server
        server.roles = server.roles.filter(r => r._id.toString() !== roleId);
        
        // Remove role from all members
        server.members.forEach(member => {
            member.roles = member.roles.filter(role => role !== roleId);
        });

        await server.save();
        return res.status(200).json({ message: 'Role deleted successfully' });
    } catch (error) {
        console.error('Server role deletion error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

export const getServerRolePermissions = async (req, res) => {
    try {
        const { id, roleId } = req.params;
        const server = await models.Server.findById(id);
        
        if (!server) {
            return res.status(404).json({ error: 'Server not found' });
        }

        const role = server.roles.find(r => r._id.toString() === roleId);
        if (!role) {
            return res.status(404).json({ error: 'Role not found' });
        }

        return res.status(200).json(role.permissions);
    } catch (error) {
        console.error('Server role permissions retrieval error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

export const updateServerRolePermissions = async (req, res) => {
    try {
        const { id, roleId } = req.params;
        const userId = req.user.id;
        const { permissions } = req.body;

        const server = await models.Server.findById(id);
        if (!server) {
            return res.status(404).json({ error: 'Server not found' });
        }

        const role = server.roles.find(r => r._id.toString() === roleId);
        if (!role) {
            return res.status(404).json({ error: 'Role not found' });
        }

        // Check if user has permission to update role permissions
        const member = server.members.find(m => m.userId.toString() === userId);
        if (!member || !member.roles.some(role => ['owner', 'admin'].includes(role))) {
            return res.status(403).json({ error: 'Insufficient permissions' });
        }

        role.permissions = permissions;
        await server.save();

        return res.status(200).json(role.permissions);
    } catch (error) {
        console.error('Server role permissions update error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

/**====================================
 * Server Channel Methods
 ======================================*/

export const getServerChannels = async (req, res) => {
    try {
        const server = await models.Server.findById(req.params.id);
        return res.status(200).json(server);
    } catch (error) {
        console.error('Server channels retrieval error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

export const createChannel = async (req, res) => {
    try {
        const serverId = req.params.id;
        const userId = req.user.id;
        const { name, type, position, permissions } = req.body;

        const server = await models.Server.findById(serverId);
        if (!server) {
            return res.status(404).json({ error: 'Server not found' });
        }

        // Check if user has permission to create channels
        const member = server.members.find(m => m.userId.toString() === userId);
        if (!member || !member.roles.some(role => ['owner', 'admin', 'moderator'].includes(role))) {
            return res.status(403).json({ error: 'Insufficient permissions' });
        }

        // Validate channel name
        if (!name || name.length < 2 || name.length > 100) {
            return res.status(400).json({ error: 'Channel name must be between 2 and 100 characters' });
        }

        // Check if channel name already exists
        if (server.channels.some(channel => channel.name === name)) {
            return res.status(400).json({ error: 'Channel name already exists' });
        }

        const channel = {
            name,
            type: type || 'text',
            position: position || server.channels.length,
            permissions: permissions || [],
            createdAt: new Date(),
            createdBy: userId
        };

        server.channels.push(channel);
        await server.save();

        return res.status(201).json(channel);
    } catch (error) {
        console.error('Server channel creation error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

export const updateChannel = async (req, res) => {
    try {
        const { id, channelId } = req.params;
        const userId = req.user.id;
        const { name, type, position, permissions } = req.body;

        const server = await models.Server.findById(id);
        if (!server) {
            return res.status(404).json({ error: 'Server not found' });
        }

        const channel = server.channels.find(c => c._id.toString() === channelId);
        if (!channel) {
            return res.status(404).json({ error: 'Channel not found' });
        }

        // Check if user has permission to update channels
        const member = server.members.find(m => m.userId.toString() === userId);
        if (!member || !member.roles.some(role => ['owner', 'admin'].includes(role))) {
            return res.status(403).json({ error: 'Insufficient permissions' });
        }

        if (name !== undefined) {
            if (name.length < 2 || name.length > 100) {
                return res.status(400).json({ error: 'Channel name must be between 2 and 100 characters' });
            }
            if (server.channels.some(c => c.name === name && c._id.toString() !== channelId)) {
                return res.status(400).json({ error: 'Channel name already exists' });
            }
            channel.name = name;
        }

        if (type !== undefined) channel.type = type;
        if (position !== undefined) channel.position = position;
        if (permissions !== undefined) channel.permissions = permissions;

        await server.save();
        return res.status(200).json(channel);
    } catch (error) {
        console.error('Server channel update error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

export const deleteChannel = async (req, res) => {
    try {
        const { id, channelId } = req.params;
        const userId = req.user.id;

        const server = await models.Server.findById(id);
        if (!server) {
            return res.status(404).json({ error: 'Server not found' });
        }

        const channel = server.channels.find(c => c._id.toString() === channelId);
        if (!channel) {
            return res.status(404).json({ error: 'Channel not found' });
        }

        // Check if user has permission to delete channels
        const member = server.members.find(m => m.userId.toString() === userId);
        if (!member || !member.roles.some(role => ['owner', 'admin'].includes(role))) {
            return res.status(403).json({ error: 'Insufficient permissions' });
        }

        // Remove channel from server
        server.channels = server.channels.filter(c => c._id.toString() !== channelId);
        await server.save();

        return res.status(200).json({ message: 'Channel deleted successfully' });
    } catch (error) {
        console.error('Server channel deletion error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

export const getServerChannelMessages = async (req, res) => {
    try {
        const { id, channelId } = req.params;
        const { limit = 50, before } = req.query;

        const server = await models.Server.findById(id);
        if (!server) {
            return res.status(404).json({ error: 'Server not found' });
        }

        const channel = server.channels.find(c => c._id.toString() === channelId);
        if (!channel) {
            return res.status(404).json({ error: 'Channel not found' });
        }

        let query = { channelId };
        if (before) {
            query.createdAt = { $lt: new Date(before) };
        }

        const messages = await models.Message.find(query)
            .sort({ createdAt: -1 })
            .limit(parseInt(limit))
            .populate('author', 'username avatar');

        return res.status(200).json(messages);
    } catch (error) {
        console.error('Server channel messages retrieval error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

export const createServerChannelMessage = async (req, res) => {
    try {
        const { id, channelId } = req.params;
        const userId = req.user.id;
        const { content, attachments } = req.body;

        const server = await models.Server.findById(id);
        if (!server) {
            return res.status(404).json({ error: 'Server not found' });
        }

        const channel = server.channels.find(c => c._id.toString() === channelId);
        if (!channel) {
            return res.status(404).json({ error: 'Channel not found' });
        }

        // Check if user has permission to send messages
        const member = server.members.find(m => m.userId.toString() === userId);
        if (!member) {
            return res.status(403).json({ error: 'You must be a member to send messages' });
        }

        const message = await models.Message.create({
            channelId,
            author: userId,
            content,
            attachments,
            createdAt: new Date()
        });

        await message.populate('author', 'username avatar');
        return res.status(201).json(message);
    } catch (error) {
        console.error('Server channel message creation error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

export const updateServerChannelMessage = async (req, res) => {
    try {
        const { id, channelId, messageId } = req.params;
        const userId = req.user.id;
        const { content, attachments } = req.body;

        const server = await models.Server.findById(id);
        if (!server) {
            return res.status(404).json({ error: 'Server not found' });
        }

        const channel = server.channels.find(c => c._id.toString() === channelId);
        if (!channel) {
            return res.status(404).json({ error: 'Channel not found' });
        }

        const message = await models.Message.findById(messageId);
        if (!message) {
            return res.status(404).json({ error: 'Message not found' });
        }

        // Check if user is the message author or has admin permissions
        const member = server.members.find(m => m.userId.toString() === userId);
        const isAuthor = message.author.toString() === userId;
        const isAdmin = member && member.roles.some(role => ['owner', 'admin', 'moderator'].includes(role));

        if (!isAuthor && !isAdmin) {
            return res.status(403).json({ error: 'Insufficient permissions' });
        }

        if (content !== undefined) message.content = content;
        if (attachments !== undefined) message.attachments = attachments;
        message.editedAt = new Date();

        await message.save();
        await message.populate('author', 'username avatar');

        return res.status(200).json(message);
    } catch (error) {
        console.error('Server channel message update error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

export const deleteServerChannelMessage = async (req, res) => {
    try {
        const { id, channelId, messageId } = req.params;
        const userId = req.user.id;

        const server = await models.Server.findById(id);
        if (!server) {
            return res.status(404).json({ error: 'Server not found' });
        }

        const channel = server.channels.find(c => c._id.toString() === channelId);
        if (!channel) {
            return res.status(404).json({ error: 'Channel not found' });
        }

        const message = await models.Message.findById(messageId);
        if (!message) {
            return res.status(404).json({ error: 'Message not found' });
        }

        // Check if user is the message author or has admin permissions
        const member = server.members.find(m => m.userId.toString() === userId);
        const isAuthor = message.author.toString() === userId;
        const isAdmin = member && member.roles.some(role => ['owner', 'admin', 'moderator'].includes(role));

        if (!isAuthor && !isAdmin) {
            return res.status(403).json({ error: 'Insufficient permissions' });
        }

        await message.deleteOne();
        return res.status(200).json({ message: 'Message deleted successfully' });
    } catch (error) {
        console.error('Server channel message deletion error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

export const getServerChannelMessageReactions = async (req, res) => {
    try {
        const { id, channelId, messageId } = req.params;

        const server = await models.Server.findById(id);
        if (!server) {
            return res.status(404).json({ error: 'Server not found' });
        }

        const channel = server.channels.find(c => c._id.toString() === channelId);
        if (!channel) {
            return res.status(404).json({ error: 'Channel not found' });
        }

        const message = await models.Message.findById(messageId);
        if (!message) {
            return res.status(404).json({ error: 'Message not found' });
        }

        return res.status(200).json(message.reactions);
    } catch (error) {
        console.error('Server channel message reactions retrieval error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

export const createServerChannelMessageReaction = async (req, res) => {
    try {
        const { id, channelId, messageId } = req.params;
        const userId = req.user.id;
        const { emoji } = req.body;

        const server = await models.Server.findById(id);
        if (!server) {
            return res.status(404).json({ error: 'Server not found' });
        }

        const channel = server.channels.find(c => c._id.toString() === channelId);
        if (!channel) {
            return res.status(404).json({ error: 'Channel not found' });
        }

        const message = await models.Message.findById(messageId);
        if (!message) {
            return res.status(404).json({ error: 'Message not found' });
        }

        // Check if user is a member
        const member = server.members.find(m => m.userId.toString() === userId);
        if (!member) {
            return res.status(403).json({ error: 'You must be a member to react to messages' });
        }

        // Add or update reaction
        const existingReaction = message.reactions.find(r => r.emoji === emoji);
        if (existingReaction) {
            if (!existingReaction.users.includes(userId)) {
                existingReaction.users.push(userId);
                existingReaction.count++;
            }
        } else {
            message.reactions.push({
                emoji,
                users: [userId],
                count: 1
            });
        }

        await message.save();
        return res.status(200).json(message.reactions);
    } catch (error) {
        console.error('Server channel message reaction creation error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

export const deleteServerChannelMessageReaction = async (req, res) => {
    try {
        const { id, channelId, messageId } = req.params;
        const userId = req.user.id;
        const { emoji } = req.body;

        const server = await models.Server.findById(id);
        if (!server) {
            return res.status(404).json({ error: 'Server not found' });
        }

        const channel = server.channels.find(c => c._id.toString() === channelId);
        if (!channel) {
            return res.status(404).json({ error: 'Channel not found' });
        }

        const message = await models.Message.findById(messageId);
        if (!message) {
            return res.status(404).json({ error: 'Message not found' });
        }

        // Remove reaction
        const reaction = message.reactions.find(r => r.emoji === emoji);
        if (reaction) {
            const userIndex = reaction.users.indexOf(userId);
            if (userIndex !== -1) {
                reaction.users.splice(userIndex, 1);
                reaction.count--;
                if (reaction.count === 0) {
                    message.reactions = message.reactions.filter(r => r.emoji !== emoji);
                }
            }
        }

        await message.save();
        return res.status(200).json(message.reactions);
    } catch (error) {
        console.error('Server channel message reaction deletion error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
}

/**====================================
 * Server Message Methods
 ======================================*/
