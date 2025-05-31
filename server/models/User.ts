import mongoose, { Schema, Document, Model } from 'mongoose';

export interface IUser extends Document {
  username: string;
  email: string;
  password: string;
  avatarUrl?: string;
  bio?: string;
  status?: string;
  createdAt: Date;
}

const UserSchema: Schema<IUser> = new Schema({
  username: { type: String, required: true, unique: true, trim: true },
  email: { type: String, required: true, unique: true, trim: true },
  password: { type: String, required: true },
  avatarUrl: { type: String },
  bio: { type: String },
  status: { type: String },
  createdAt: { type: Date, default: Date.now }
});

export default (mongoose.models.User as Model<IUser>) || mongoose.model<IUser>('User', UserSchema); 